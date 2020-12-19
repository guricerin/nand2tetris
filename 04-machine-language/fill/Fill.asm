// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed.
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

    // lim = 24575 (screen's memory map end)
    @24575
    D=A
    @lim
    M=D

(LISTENER)
    // addr = 16384 (screen's base address)
    @SCREEN
    D=A
    @addr
    M=D

    // if KBD = 0 then goto CLEAR
    // else goto FILL
    @KBD
    D=M
    @CLEAR
    D;JEQ

(FILL)
    // if addr - lim > 0 then goto LISTENER
    @addr
    D=M
    @lim
    D=D-M // addr - lim
    @LISTENER
    D;JGT

    // RAM[addr] = [1; 16]
    @addr
    A=M
    M=-1

    // addr++
    @addr
    M=M+1

    @FILL
    0;JMP

(CLEAR)
    // if addr - lim > 0 then goto LISTENER
    @addr
    D=M
    @lim
    D=D-M // addr - lim
    @LISTENER
    D;JGT

    // RAM[addr] = [0; 16]
    @addr
    A=M
    M=0

    // addr++
    @addr
    M=M+1

    @CLEAR
    0;JMP
