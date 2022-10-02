# Extremely high performance stun server

## feature
- Extremely high performance
- Pure rust


## examples

```bash
cargo run --example single_thread

cargo run --example mutli_threads

```


Linux
```bash 
cargo run --example mutli_threads_recv_send_mmsg 
```


## Pressure test

machine information:
```
Architecture:          x86_64
CPU op-mode(s):        32-bit, 64-bit
Byte Order:            Little Endian
CPU(s):                32
On-line CPU(s) list:   0-31
Thread(s) per core:    2
Core(s) per socket:    8
Socket(s):             2
NUMA node(s):          2
Vendor ID:             GenuineIntel
CPU family:            6
Model:                 79
Model name:            Intel(R) Xeon(R) CPU E5-2620 v4 @ 2.10GHz
Stepping:              1
CPU MHz:               2298.761
CPU max MHz:           3000.0000
CPU min MHz:           1200.0000
BogoMIPS:              4191.38
Virtualization:        VT-x
L1d cache:             32K
L1i cache:             32K
L2 cache:              256K
L3 cache:              20480K
NUMA node0 CPU(s):     0-7,16-23
NUMA node1 CPU(s):     8-15,24-31
```

```
02:00.0 Ethernet controller: Intel Corporation 82599ES 10-Gigabit SFI/SFP+ Network Connection (rev 01)
02:00.1 Ethernet controller: Intel Corporation 82599ES 10-Gigabit SFI/SFP+ Network Connection (rev 01)
```

- one machine with 2 Ethernet cards power by 'Debian GNU/Linux 8 (jessie)' os can receive 2 million packages per second.

