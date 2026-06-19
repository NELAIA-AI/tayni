// Minimal HTTP server for benchmark comparison - Go
// Compile with: go build -ldflags="-s -w" -o go_http.exe go_http.go
package main

import (
	"fmt"
	"net/http"
	"os"
)

func main() {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"benchmark":"go","ok":1}`))
		go func() { os.Exit(0) }() // Exit after one request
	})
	fmt.Println("Go HTTP Server listening on port 38083...")
	http.ListenAndServe("127.0.0.1:38083", nil)
}
