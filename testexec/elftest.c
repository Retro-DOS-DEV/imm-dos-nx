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

static strptr path_ptr;

int open_file(char *path) {
  int length;
  for (length = 0; length < 255; length++) {
    if (path[length] == 0) {
      break;
    }
  }

  path_ptr.addr = (int) path;
  path_ptr.length = length;
  return syscall(0x10, (int)(&path_ptr), 0, 0);
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

int fork() {
  return syscall(1, 0, 0, 0);
}

void yield() {
  syscall(6, 0, 0, 0);
}

void sleep(int ms) {
  syscall(5, ms, 0, 0);
}

void _start() {
  //write_file(handle, "HELLO FROM ELF");
  int id = fork();
  int handle = open_file("DEV:\\TTY1");
  while (1) {
    if (id == 0) {
      write_file(handle, "TOCK ");
    } else {
      write_file(handle, "TICK ");
    }
    sleep(1000);
  }
}