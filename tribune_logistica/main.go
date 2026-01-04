package main

import (
	"fmt"
	"log"
	"os"

	"github.com/book_legion-tribune_logistica/internal/epub" // replace with actual module path
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run dumb_test.go /path/to/book.epub")
		return
	}

	epubPath := os.Args[1]

	spine, err := epub.LoadSpine(epubPath)
	if err != nil {
		log.Fatalf("Failed to load spine: %v", err)
	}

	fmt.Printf("Loaded spine with %d items:\n", len(spine))
	for _, item := range spine {
		fmt.Printf(
			"Index=%d, ID=%q, Href=%q\n",
			item.Index, item.ID, item.Href,
		)
	}
}
