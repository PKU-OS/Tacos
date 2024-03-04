#include <assert.h>
#include <dirent.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/types.h>
#include <unistd.h>

/* ------------------------------- DEBUG MACRO ------------------------------ */
#ifdef NDEBUG
#define DEBUG_PRINTF(...)                                                      \
  do {                                                                         \
  } while (0)
#else
#define DEBUG_PRINTF(...)                                                      \
  do {                                                                         \
    fprintf(stderr, __VA_ARGS__);                                              \
  } while (0)
#endif

/* ------------------------------ HELPER MACROS ----------------------------- */

// Round up integer divice (a / b).
#define ROUNDUP(n, div) (((n) + (div) - 1) / (div))
// Add freemap, root and swap to file number.
#define TOTAL_FILE_NUM(OBJ_FILE_NUM) ((OBJ_FILE_NUM) + 2 + 1)

/* -------------------------------- CONSTANTS ------------------------------- */

#define SECTOR_SIZE       512
#define FILE_NAME_LEN_MAX 28
#define MAX_FILES         200
// Inode magic number.
#define MAGIC             0x494e4f44
// 10MiB disk.
#define DISK_SIZE         (10 << 20)
// Total sector number.
#define SECTOR_NUM        ROUNDUP(DISK_SIZE, SECTOR_SIZE)
#define FREEMAP_BITS      SECTOR_NUM
// Used after.
#define FREEMAP_BYTES     ROUNDUP(FREEMAP_BITS, 8)
// Sectors of freemap.
#define FREEMAP_SECTORS   ROUNDUP(FREEMAP_BYTES,  SECTOR_SIZE)
// 4MiB swap.
#define SWAP_SPACE        (4 << 20)
// Add another FREE_NUMBER inodes could be used to create new files.
#define FREE_NUMBER       10

const uint32_t FREE_MAP_SECTOR = 0;
const uint32_t ROOT_DIR_SECTOR = 1;

const char SWAP_FNAME[] = ".glbswap";
const char DISK_FILENAME[] = "disk.img";
const char TEST_DIR[] = "user";

/* --------------------------------- STRUCT --------------------------------- */
struct inner_inode {
  uint32_t start;
  uint32_t len;
  uint32_t magic;
};

struct ondisk_inode {
  struct inner_inode inner;
  uint8_t unused[SECTOR_SIZE - sizeof(struct inner_inode)];
};

struct dentry {
  char name[FILE_NAME_LEN_MAX];
  uint32_t inum;
};

/* ---------------------------------- IMPL ---------------------------------- */

static char filenames[MAX_FILES][FILE_NAME_LEN_MAX];
static FILE* files[MAX_FILES];
static uint32_t FILE_NUMBER = 0;

const char* last_slash(const char* str) {
  for (int i = strlen(str) - 1; i >= 0; i--) {
    if (str[i] == '/') {
      return &str[i];
    }
  }

  return NULL;
}

int ends_with(const char *str, const char *suffix) {
  size_t str_len = strlen(str);
  size_t suffix_len = strlen(suffix);

  if (str_len < suffix_len) {
    return 0;
  }

  const char *str_end = str + (str_len - suffix_len);
  return strcmp(str_end, suffix) == 0;
}

int should_copy(const char* path) {
  return strstr(path, ".") == NULL ||
         ends_with(path, "sample.txt") ||
         ends_with(path, "zeros");
}

char **read_filenames(const char* dir) {
  assert(FILE_NUMBER == 0);

  char command[128] = {0};
  char path[128] = {0};
  snprintf(command, sizeof(command), "find %s -type f", dir);

  FILE* fp = popen(command, "r");
  if (fp == NULL) {
    perror("popen");
  }

  while (fgets(path, sizeof(path) - 1, fp) != NULL) {
    // Read the full path of files that should be copied to disk.img.
    // DEBUG_PRINTF("%s", path);
    size_t len = strcspn(path, "\n");
    if (path[len] == '\n') {
      path[len] = '\0'; // Replace the newline with null terminator
    }

    if (should_copy(path)) {
      files[FILE_NUMBER] = fopen(path, "rb");
      if (!files[FILE_NUMBER]) {
        DEBUG_PRINTF("executable %s not found.\n", path);
        exit(1);
      }

      const char* name = last_slash(path);
      if (name == NULL) {
        perror("last slash!");
      }
      strcpy(filenames[FILE_NUMBER++], name+1);
    }
  }

  return (char**)filenames;
}

void free_map_set(uint8_t *free_map, int idx) {
  free_map[idx / 8] |= 1 << (idx % 8);
}

size_t get_file_size(FILE* fp) {
    fseek(fp, 0, SEEK_END);
    size_t ret = ftell(fp);
    rewind(fp);
    return ret;
}

void make_disk_img(FILE *disk, FILE **files) {
  ftruncate(fileno(disk), DISK_SIZE);
  DEBUG_PRINTF(
    "Make a disk with %uKiB\n"
    "Freemap inum = %u\n"
    "Rootdir inum = %u\n"
    "Inode range = [%u, %u)\n",
    DISK_SIZE / 1024,
    FREE_MAP_SECTOR, ROOT_DIR_SECTOR,
    2, TOTAL_FILE_NUM(FILE_NUMBER));

  // Make freemap, the first file.
  // However, write it to disk latter.
  uint32_t free_map_content_start = TOTAL_FILE_NUM(FILE_NUMBER);
  struct inner_inode inner_free_map = {.len = FREEMAP_BYTES,
                              .start = free_map_content_start,
                              .magic = MAGIC};
  struct ondisk_inode free_map_inode = {.inner = inner_free_map, .unused = {0}};
  uint8_t free_map[FREEMAP_BYTES] = {0};
  DEBUG_PRINTF("Freemap: [%u, %u), len = %u\n",
    free_map_content_start,
    free_map_content_start + ROUNDUP(inner_free_map.len, SECTOR_SIZE),
    inner_free_map.len);

  // Make root DIR. The second file. Include swap file in root.
  uint32_t root_content_start = free_map_content_start + ROUNDUP(inner_free_map.len, SECTOR_SIZE);
  uint32_t root_map_size = FILE_NUMBER + 1 + FREE_NUMBER;
  uint32_t root_content_len = root_map_size * sizeof(struct dentry);
  struct inner_inode inner_root_dir = {.len = root_content_len,
                              .start = root_content_start,
                              .magic = MAGIC};
  struct ondisk_inode root_dir_inode = {.inner = inner_root_dir, .unused = {0}};
  DEBUG_PRINTF("Root dir: [%u, %u), len = %u\n",
    root_content_start,
    root_content_start + ROUNDUP(inner_root_dir.len, SECTOR_SIZE),
    inner_root_dir.len);

  // Write root DIR inode.
  fseek(disk, ROOT_DIR_SECTOR * SECTOR_SIZE, SEEK_SET);
  fwrite(&root_dir_inode, sizeof(root_dir_inode), 1, disk);

  // Make content of root DIR, including swap file.
  struct dentry *root_dir_content = (struct dentry *)calloc(root_map_size, sizeof(struct dentry));
  for (uint32_t i = 0; i < FILE_NUMBER; i++) {
    strncpy(root_dir_content[i].name, filenames[i], FILE_NAME_LEN_MAX);
    root_dir_content[i].inum = i + 2;
    DEBUG_PRINTF("Add %s to root dir, inum = %u\n", filenames[i], i + 2);
  }
  strncpy(root_dir_content[FILE_NUMBER].name, SWAP_FNAME, FILE_NAME_LEN_MAX);
  root_dir_content[FILE_NUMBER].inum = FILE_NUMBER + 2;
  DEBUG_PRINTF("Add %s to root dir, inum = %u\n", SWAP_FNAME, FILE_NUMBER + 2);

  // Write content of the root DIR.
  fseek(disk, root_content_start * SECTOR_SIZE, SEEK_SET);
  fwrite(root_dir_content, root_content_len, 1, disk);
  free(root_dir_content);

  // Calculate current sector number.
  uint32_t current = root_content_start + ROUNDUP(root_content_len, SECTOR_SIZE);

  // Copy file one by one.
  struct ondisk_inode file_inode;
  bzero(file_inode.unused, sizeof(file_inode.unused));

  for (uint32_t i = 0; i < FILE_NUMBER; i++) {
    // Read the content of the file, then write it to disk.
    fseek(disk, current * SECTOR_SIZE, SEEK_SET);
    FILE *f = files[i];
    size_t size = get_file_size(f);
    void* buf = malloc(size);
    assert(fread(buf, 1, size, f) == size);
    fwrite(buf, 1, size, disk);
    free(buf);

    // Write the inode.
    file_inode.inner.len = size;
    file_inode.inner.start = current;
    file_inode.inner.magic = MAGIC;
    fseek(disk, (i + 2) * SECTOR_SIZE, SEEK_SET);
    fwrite(&file_inode, sizeof(file_inode), 1, disk);

    current += ROUNDUP(size, SECTOR_SIZE);
    DEBUG_PRINTF("FILE %s: [%u, %u), inum = %u, size = %u\n",
      filenames[i],
      file_inode.inner.start, current,
      i + 2, file_inode.inner.len);
  }
  // Make zeroed swap file.
  void* buf = calloc(SECTOR_SIZE, sizeof(uint8_t));
  fseek(disk, current * SECTOR_SIZE, SEEK_SET);
  fwrite(buf, ROUNDUP(SWAP_SPACE, SECTOR_SIZE), SECTOR_SIZE, disk);
  // Make swap inode.
  struct inner_inode inner_swap = {.len = SWAP_SPACE,
                              .start = current,
                              .magic = MAGIC};
  struct ondisk_inode swap_inode = {.inner = inner_swap, .unused = {0}};
  fseek(disk, (FILE_NUMBER + 2) * SECTOR_SIZE, SEEK_SET);
  fwrite(&swap_inode, sizeof(swap_inode), 1, disk);
  DEBUG_PRINTF("FILE %s: [%u, %u), inum = %u, size = %uKiB\n",
    SWAP_FNAME,
    current,
    current + ROUNDUP(SWAP_SPACE, SECTOR_SIZE),
    FILE_NUMBER + 2,
    SWAP_SPACE / 1024);
  current += ROUNDUP(SWAP_SPACE, SECTOR_SIZE);
  free(buf);

  // Write free map.
  for (int i = 0; i < current; i++) {
    free_map_set(free_map, i);
  }
  rewind(disk);
  fwrite(&free_map_inode, sizeof(free_map_inode), 1, disk);
  fseek(disk, free_map_content_start * SECTOR_SIZE, SEEK_SET);
  fwrite(free_map, sizeof(free_map), 1, disk);
  DEBUG_PRINTF("Freemap written\n");
}

int main(int argc, char* argv[]) {
  // Read filenames.
  read_filenames(TEST_DIR);

  // Open diskfile.
  FILE *disk_file = fopen(DISK_FILENAME, "w+");
  if (disk_file == NULL) {
    perror("disk file");
    exit(1);
  }

  // Make disk img.
  make_disk_img(disk_file, files);

  // Free resources.
  for (int i = 0; i < FILE_NUMBER; i++) {
    fclose(files[i]);
  }

  return 0;
}
