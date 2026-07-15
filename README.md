# peek

A fast TCP port scanner written in Rust.

`peek` is a lightweight command-line TCP scanner that uses non-blocking sockets and `poll()` for concurrent port scanning without async runtimes.

The project was created to explore low-level networking concepts in Rust:
- TCP connections
- sockets and file descriptors
- non-blocking I/O
- event-driven programming
- connection timeouts

---

## Features

- TCP port scanning
- Concurrent non-blocking connections
- Custom timeout handling
- Hostname/IP support
- Port range scanning
- Colored terminal output
- Scan statistics
- Verbose output mode

---

## Installation

Clone the repository:

```bash
git clone https://github.com/yourname/peek.git
cd peek
```

---

## Usage

Scan ports:

```bash
peek www.google.com 0-10000
```

Example output:

peek - TCP Scanner

Target: google.com
IP: 142.250.120.102

PORT     STATE
---------------
80       OPEN
443      OPEN

Scan finished in 1.352s

✔ Open ports:      2
✘ Closed ports:    1
? Timeout ports:   997

## Vebose mode
Show all scanned ports
Example:

PORT     STATE
---------------
22       CLOSED
53       CLOSED
80       OPEN
443      OPEN

---

## Technologies
- Rust
- nix crate
- clap
- owo-colors

---
                             >=>      
                             >=>      
>=> >=>    >==>      >==>    >=>  >=> 
>>   >=> >>   >=>  >>   >=>  >=> >=>  
>>   >=> >>===>>=> >>===>>=> >=>=>    
>=> >=>  >>        >>        >=> >=>  
>=>       >====>    >====>   >=>  >=> 
>=>                                   
