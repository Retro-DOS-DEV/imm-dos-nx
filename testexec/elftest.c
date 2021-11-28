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

void terminate(int code) {
  syscall(0, code, 0, 0);
}

void wait(int id) {
  int status;
  syscall(9, id, (int)(&status), 0);
}

void _start() {
  int id = fork();
  int handle = open_file("DEV:\\TTY1");
  if (id == 0) {
    // child process
    write_file(handle, "  Child running\n");
    sleep(5000);
    write_file(handle, "  Child done\n");
    terminate(1);
  } else {
    // parent
    write_file(handle, "Wait for child\n");
    wait(id);
    write_file(handle, "Child returned.");
    yield();
  }
  terminate(0);
}