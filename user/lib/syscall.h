#define SYS_HALT 1   /**< Halt the operating system. */
#define SYS_EXIT 2   /**< Terminate this process. */
#define SYS_EXEC 3   /**< Start another process. */
#define SYS_WAIT 4   /**< Wait for a child process to die. */
#define SYS_REMOVE 5 /**< Delete a file. */
#define SYS_OPEN 6   /**< Open a file. */
#define SYS_READ 7   /**< Read from a file. */
#define SYS_WRITE 8  /**< Write to a file. */
#define SYS_SEEK 9   /**< Change position in a file. */
#define SYS_TELL 10  /**< Report current position in a file. */
#define SYS_CLOSE 11 /**< Close a file. */
#define SYS_FSTAT 12 /**< Get metadata about a file */

/* Project 3 and optionally project 4. */
#define SYS_MMAP 13   /**< Map a file into memory. */
#define SYS_MUNMAP 14 /**< Remove a memory mapping. */

/* Project 4 only. */
#define SYS_CHDIR 15 /**< Change the current directory. */
#define SYS_MKDIR 16 /**< Create a directory. */
