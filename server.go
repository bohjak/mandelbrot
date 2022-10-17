package main

import (
	"fmt"
	"log"
	"net/http"
	"sync"
)

type connection struct {
	writer  http.ResponseWriter
	flusher http.Flusher
}

type broker struct {
	connections      map[string]*connection
	connectionsMutex sync.RWMutex
}

// subscribes the requestee to the event stream
func (b *broker) handleConnection(rw http.ResponseWriter, req *http.Request) {
	// if I understand this correctly, this basically saves the ResponseWriter as a flusher if it implements that interface
	flusher, ok := rw.(http.Flusher)
	if !ok {
		http.Error(rw, "streaming unsupported", http.StatusInternalServerError)
		return
	}

	// register new connection
	b.connectionsMutex.Lock()
	b.connections[req.RemoteAddr] = &connection{
		writer:  rw,
		flusher: flusher,
	}
	b.connectionsMutex.Unlock()

	rw.Header().Set("Content-Type", "text/event-stream")
	rw.Header().Set("Cache-Control", "no-cache")
	rw.Header().Set("Connection", "keep-alive")
	rw.Header().Set("Access-Control-Allow-Origin", "*")

	// wait for the connection to be closed, then remove it
	<-req.Context().Done()
	b.removeConnection(req.RemoteAddr)
}

func (b *broker) removeConnection(key string) {
	b.connectionsMutex.Lock()
	delete(b.connections, key)
	b.connectionsMutex.Unlock()
}

// sends the reload event to all subscribed connections
func (b *broker) sendReload() {
	b.connectionsMutex.RLock()
	defer b.connectionsMutex.RUnlock()

	// data is mandatory
	msgBytes := []byte("data: reload\n\n")
	for client, connection := range b.connections {
		_, err := connection.writer.Write(msgBytes)
		if err != nil {
			b.removeConnection(client)
			continue
		}

		connection.flusher.Flush()
	}
}

// basically just a wrapper for broker.sendReload()
func (b *broker) handleReloadReq(res http.ResponseWriter, req *http.Request) {
	b.sendReload()
}

func main() {
	// serve the current directory at '/'
	fs := http.FileServer(http.Dir("."))
	http.Handle("/", fs)

	broker := broker{connections: map[string]*connection{}}
	// endpoint for subscribing new connections to the event stream
	http.HandleFunc("/sse", broker.handleConnection)
	// endpoint for triggering sending the reload event
	http.HandleFunc("/sse/reload", broker.handleReloadReq)

	fmt.Println("starting on http://localhost:3000")
	fmt.Println("send requests to http://localhost:3000/sse/reload to trigger reloads")
	log.Fatal(http.ListenAndServe(":3000", nil))
}
