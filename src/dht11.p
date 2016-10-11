.origin 0
.entrypoint start

#define CPRUCFG  c4

#define GPIO0 0x44E07000
#define GPIO1 0x4804C000
#define GPIO2 0x481AC000
#define GPIO3 0x481AE000

#define GPIO_OE 0x134
#define GPIO_DATAIN 0x138
#define GPIO_CLEARDATAOUT 0x190
#define GPIO_SETDATAOUT 0x194

#define SIGNAL_PIN 17
#define DEBUG_PIN 18
#define ONE_US    100
#define ONE_MS    100 * 1000

// Address for the Constant table Programmable Pointer Register 0(CTPPR_0)
#define CTPPR_0         0x22028


#define PRU0_ARM_INTERRUPT      19


// PINS
// P9_23 gpio1[17] 0x844

// registers:
// common:
    // r8: delay time
// when writing:
    // r2: signal pin mask
    // r3: pull up register
    // r4: pull down register
// when reading:
    // r2: read data
    // r3: read address
    // r4: bitcount
    // r5: checksum
    // r6: timeout
    // r7: result

.macro ST32
.mparam src,dst
    SBBO    src,dst,#0x00,4
.endm


start:
    // enable OCP master port
    LBCO r0, CPRUCFG, 4, 4
    CLR  r0.t4
    SBCO r0, CPRUCFG, 4, 4

    // set SIGNAL_PIN direction to OUT
    MOV r0, GPIO1 | GPIO_OE
    LBBO r1, r0, 0, 4
    CLR r1, r1, SIGNAL_PIN
    SBBO r1, r0, 0, 4

    // set DEBUG_PIN direction to OUT
    MOV r0, GPIO1 | GPIO_OE
    LBBO r1, r0, 0, 4
    CLR r1, r1, DEBUG_PIN
    SBBO r1, r0, 0, 4

    // Configure the programmable pointer register for PRU0 by setting c28_pointer[15:0]
    // field to 0x0120.  This will make C28 point to 0x00012000 (PRU shared RAM).
    MOV     r0, 0x00000120
    MOV       r1, CTPPR_0
    ST32      r0, r1

    // clear shared memory
    MOV       r0, 0
    SBCO      r0, c28, 0, 8
    SBCO      r0, c28, 4, 8


    MOV r2, 1 << SIGNAL_PIN
    MOV r9, 1 << DEBUG_PIN
    MOV r3, GPIO1 | GPIO_SETDATAOUT
    MOV r4, GPIO1 | GPIO_CLEARDATAOUT

    SBBO r9, r4, 0, 4

    // pull up: (> 18ms) ask for data
    SBBO r2, r4, 0, 4
    MOV r8, 20 * ONE_MS
    CALL delay

    // pull down: (20-40us) starts transfer
    SBBO r2, r3, 0, 4
    MOV r8, 20 * ONE_US
    CALL delay

    // set SIGNAL_PIN direction to IN
    MOV r0, GPIO1 | GPIO_OE
    LBBO r1, r0, 0, 4
    SET r1, r1, SIGNAL_PIN
    SBBO r1, r0, 0, 4

    MOV r6, 0
wait_pull_down:
    MOV r3, GPIO1 | GPIO_DATAIN
    LBBO r2, r3, 0, 4
    QBBC on_pull_down, r2, SIGNAL_PIN
    ADD r6, r6, 1
    // time out after 20.48us since the total pull down
    // timeout is 40us
    LSR r8, r6, 8
    QBLT response_timeout, r8, 8
    QBA wait_pull_down

on_pull_down:
    MOV r6, 0
wait_pull_up:
    MOV r3, GPIO1 | GPIO_DATAIN
    LBBO r2, r3, 0, 4
    QBBS on_data, r2, SIGNAL_PIN
    ADD r6, r6, 1
    // time out after 81.92us
    LSR r8, r6, 8
    QBLT response_timeout, r8, 32
    QBA wait_pull_up

response_timeout:
    // let's try again
    // MOV r8, 5000 * ONE_MS
    // CALL delay
    // QBA start
    MOV       r31.b0, PRU0_ARM_INTERRUPT+16
    HALT

on_data:
    MOV r5, 0
    MOV r4, 0
    MOV r6, 0
    MOV r7, 0

wait_start_bit:
    MOV r3, GPIO1 | GPIO_DATAIN
    LBBO r2, r3, 0, 4
    QBBC on_data_bit, r2, SIGNAL_PIN
    ADD r6, r6, 1
    // time out after 81.92us
    LSR r8, r6, 8
    QBLT response_timeout, r8, 32
    QBA wait_start_bit

on_data_bit:
    MOV r6, 0
wait_data_bit:
    // wait at least 50us to begin read
    MOV r3, GPIO1 | GPIO_DATAIN
    LBBO r2, r3, 0, 4
    QBBS read_data_bit, r2, SIGNAL_PIN
    ADD r6, r6, 1
    // time out after 53.76us
    LSR r8, r6, 8
    QBLT response_timeout, r8, 21
    QBA wait_data_bit

read_data_bit:
    MOV r6, 0
    CALL debug_on
count_bit:
    MOV r3, GPIO1 | GPIO_DATAIN
    LBBO r2, r3, 0, 4
    QBBC done_bit, r2, SIGNAL_PIN
    // time out after 71.68us
    LSR r8, r6, 8
    QBLT response_timeout, r8, 28
    ADD r6, r6, 1
    QBA count_bit

done_bit:
    CALL debug_off

    // is bit 0 if r6 / 128 <= 0x80
    // is bit 1 if r6 / 128 > 0x80
    QBGE bit_zero, r6, 0x80
    QBGT r7_set_bit, r4, 32
    OR r5, r5, 1
    QBA bit_zero
r7_set_bit:
    OR r7, r7, 1
bit_zero:
    ADD r4, r4, 1    
    QBGT r7_read_next_bit, r4, 32
    QBGT r5_read_next_bit, r4, 40

    // save value
    SBCO      r7, c28, 0, 8
    SBCO      r5, c28, 4, 8
    MOV       r31.b0, PRU0_ARM_INTERRUPT+16
    HALT
    // MOV r8, 5000 * ONE_MS
    // CALL delay
    // QBA start

r7_read_next_bit:
    LSL r7, r7, 1
    QBA wait_start_bit

r5_read_next_bit:
    LSL r5, r5, 1
    QBA wait_start_bit

delay:
    SUB r8, r8, 1
    QBNE delay, r8, 0
    RET

debug_on:
    MOV r10, GPIO1 | GPIO_SETDATAOUT
    SBBO r9, r10, 0, 4
    RET

debug_off:
    MOV r10, GPIO1 | GPIO_CLEARDATAOUT
    SBBO r9, r10, 0, 4
    RET