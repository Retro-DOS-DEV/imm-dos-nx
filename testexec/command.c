// Simple shell for running executables

typedef struct strptr {
  int addr;
  int length;
} strptr;

typedef struct dir_entry {
  char file_name[8];
  char file_ext[3];
  unsigned short file_type;
  unsigned int byte_size;
} dir_entry;

const int stdin = 0;
const int stdout = 1;

int syscall(int method, int arg0, int arg1, int arg2) {
  register int eax asm ("eax") = method;
  register int ebx asm ("ebx") = arg0;
  register int ecx asm ("ecx") = arg1;
  register int edx asm ("edx") = arg2;
  asm volatile (
    "int $0x2b"
    : "=r"(eax)
    : "r"(eax), "r"(ebx), "r"(ecx), "r"(edx)
  );
  return eax;
}

int strlen(char *buffer) {
  for (int i = 0; i < 0xffffffff; i++) {
    if (buffer[i] == 0) {
      return i;
    }
  }
  return 0;
}

int write_file(int handle, char *buffer) {
  int length;
  for (length = 0; length < 255; length++) {
    if (buffer[length] == 0) {
      break;
    }
  }
  return syscall(0x13, handle, (int)(buffer), length);
}

int read_file(int handle, char *buffer, int max) {
  return syscall(0x12, handle, (int)(buffer), max);
}

int open_dir(char *path) {
  strptr path_ptr = {
    .addr = (int) path,
    .length = strlen(path),
  };
  return syscall(0x1a, (int)(&path_ptr), 0, 0);
}

int read_dir(int handle, dir_entry *entry) {
  return syscall(0x1b, handle, (int)entry, 0);
}

void exec(strptr *path, int format) {
  syscall(2, (int) path, 0, format);
}

int fork() {
  return syscall(1, 0, 0, 0);
}

void yield() {
  syscall(6, 0, 0, 0);
}

void wait(int id) {
  int status;
  syscall(9, id, (int)(&status), 0);
}

void terminate(int code) {
  syscall(0, code, 0, 0);
}

int get_current_drive_name(char *buffer) {
  return syscall(0x22, (int)buffer, 0, 0) & 7;
}

int change_drive(strptr *drive_name) {
  return syscall(0x21, (int)drive_name, 0, 0);
}

static char current_drive_name[8] = { 0, 0, 0, 0, 0, 0, 0, 0 };
static int current_drive_name_length;

void write_drive_name(int handle) {
  syscall(0x13, stdout, (int)(current_drive_name), current_drive_name_length);
}

void command_cd() {
  write_file(stdout, "\nUnimplemented.\n");
}

void command_dir() {
  write_file(stdout, "  Directory of ");
  write_drive_name(stdout);
  write_file(stdout, ":\\\n\n");
  // get each directory entry
  int dir_handle = open_dir("");
  dir_entry entry;
  int has_more = 1;
  int total_files = 0;
  char dir_entry_line[15];
  for (int i = 0; i < 14; i++) {
    dir_entry_line[i] = 0x20;
  }
  dir_entry_line[14] = '\n';
  while (has_more) {
    has_more = read_dir(dir_handle, &entry);
    // print entry details
    for (int i = 0; i < 8; i++) {
      dir_entry_line[2 + i] = entry.file_name[i];
    }
    for (int i = 0; i < 3; i++) {
      dir_entry_line[11 + i] = entry.file_ext[i];
    }
    syscall(0x13, stdout, (int)(dir_entry_line), 15);
    total_files += 1;
  }
  
  // print total files
  // print total dirs
}

struct command {
  char *name;
  void *fn;
};

static struct command commands_2[] = {
  {
    .name = "cd",
    .fn = (void*)command_cd,
  },
};
static struct command commands_3[] = {
  {
    .name = "dir",
    .fn = (void*)command_dir,
  },
};

static char readbuffer[512];
static int current_drive_number = 0x80;

void run(int command_end) {
  // check for matching builtins
  struct command *command_array = 0;
  int command_array_count = 0;
  switch (command_end) {
    case 2:
      command_array = commands_2;
      command_array_count = sizeof(commands_2) / sizeof(struct command);
      break;
    case 3:
      command_array = commands_3;
      command_array_count = sizeof(commands_3) / sizeof(struct command);
      break;
  }
  if (command_array_count > 0) {
    for (int i = 0; i < command_array_count; i++) {
      int match = 1;
      for (int j = 0; j < command_end; j++) {
        if (readbuffer[j] != command_array[i].name[j]) {
          j = command_end;
          match = 0;
        }
      }
      if (match) {
        ((void (*)())(command_array[i].fn))();
        return;
      }
    }
  }

  // check for drive switch command
  if (readbuffer[command_end - 1] == ':') {
    strptr name_ptr;
    name_ptr.addr = (int) readbuffer;
    name_ptr.length = command_end - 1;

    current_drive_number = change_drive(&name_ptr);
    return;
  }

  // assume the command is an attempt to run an executable
  int id = fork();
  if (id == 0) {
    strptr path_ptr;
    path_ptr.addr = (int) readbuffer;
    path_ptr.length = command_end;

    exec(&path_ptr, 0);
    
    write_file(stdout, "\nFailed to execute program\n");
    terminate(1);
  } else {
    wait(id);
    write_file(stdout, "\nExited, resuming...\n");
  }
}

void _start() {
  while (1) {
    current_drive_name_length = get_current_drive_name(current_drive_name);
    // print drive
    write_drive_name(stdout);
    write_file(stdout, ":\\");
    // write cwd
    
    // print prompt
    write_file(stdout, "> ");
    int bytes_read = read_file(stdin, readbuffer, 512);
    int command_end;
    for (command_end = 0; command_end < bytes_read; command_end++) {
      if (readbuffer[command_end] == ' ' || readbuffer[command_end] == '\n') {
        break;
      }
    }
    run(command_end);
  }
}
