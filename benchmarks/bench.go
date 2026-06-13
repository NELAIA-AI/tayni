// High-performance HTTP benchmark tool
package main

import (
	"fmt"
	"net"
	"os"
	"strconv"
	"sync"
	"sync/atomic"
	"time"
)

func main() {
	if len(os.Args) < 4 {
		fmt.Println("Usage: bench <host:port> <requests> <concurrent>")
		os.Exit(1)
	}

	addr := os.Args[1]
	requests, _ := strconv.Atoi(os.Args[2])
	concurrent, _ := strconv.Atoi(os.Args[3])

	var success int64
	var errors int64
	var wg sync.WaitGroup

	reqPerWorker := requests / concurrent
	
	start := time.Now()

	for i := 0; i < concurrent; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := 0; j < reqPerWorker; j++ {
				conn, err := net.DialTimeout("tcp", addr, 5*time.Second)
				if err != nil {
					atomic.AddInt64(&errors, 1)
					continue
				}
				conn.Write([]byte("GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n"))
				buf := make([]byte, 4096)
				n, _ := conn.Read(buf)
				conn.Close()
				if n > 0 {
					atomic.AddInt64(&success, 1)
				} else {
					atomic.AddInt64(&errors, 1)
				}
			}
		}()
	}

	wg.Wait()
	elapsed := time.Since(start).Seconds()

	rps := float64(success) / elapsed
	fmt.Printf("Requests: %d, Errors: %d, Time: %.2fs, RPS: %.0f\n", success, errors, elapsed, rps)
}
