package main

import (
	"fmt"
	"os"

	"github.com/book_legion-tribune_logistica/internal/epub" // replace with actual module path
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run dumb_test.go /path/to/book.epub")
		return
	}

	epubPath := os.Args[1]

	epub, err := epub.New(epubPath)
	if err != nil {
		fmt.Printf("Error in epub generation: %v \n", err)
	}

	fmt.Printf("Loaded nav with %d items:\n", len(epub.Nav))
	for _, item := range epub.Nav {
		fmt.Printf(
			"Spine Index=%d, Title=%q, Chapter no.= %d\n",
			item.Index, item.Title, item.Number,
		)
	}
}
