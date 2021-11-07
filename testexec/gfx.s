.intel_syntax noprefix
.code32
.global start

start:
  # enter mode 13h
  mov eax, 0x50
  mov ebx, 0x13
  int 0x2b

  # sleep for 5 seconds
  mov eax, 5
  mov ebx, 5000
  int 0x2b

  # enter mode 3h
  mov eax, 0x50
  mov ebx, 0x03
  int 0x2b

  mov eax, 0
  int 0x2b
