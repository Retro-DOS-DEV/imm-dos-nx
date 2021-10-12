// Simple shell for running executables

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

static char readbuffer[512];

void _start() {
  // assume handles 0, 1, 2 are already established
  int stdin = 0;
  int stdout = 1;

  while (1) {
    // print prompt
    write_file(stdout, "> ");
    int bytes_read = read_file(stdin, readbuffer, 512);
  }
}