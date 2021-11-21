// Simple shell for running executables

typedef struct strptr {
  int addr;
  int length;
} strptr;

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

static char readbuffer[512];

void _start() {
  // assume handles 0, 1, 2 are already established
  int stdin = 0;
  int stdout = 1;

  char current_drive_name[8] = { 0, 0, 0, 0, 0, 0, 0, 0 };
  int current_drive_name_length;

  while (1) {
    current_drive_name_length = get_current_drive_name(current_drive_name);
    // print drive
    syscall(0x13, stdout, (int)(current_drive_name), current_drive_name_length);
    write_file(stdout, ":");
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
}