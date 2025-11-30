use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};
use std::fs::{self, File};
use regex::Regex;

pub struct Chapter {
    pub num: usize,
    pub title: String,
    pub html: String,
}

pub struct MyEpub {
    builder: EpubBuilder<ZipLibrary>,
    file: File,
}
#[allow(dead_code)]
impl MyEpub {

    pub fn new(title: &str,  author: &str) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(title)?;

        let file = File::create(format!("{}/{}.epub",title,title))?;
        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

        // metadata
        builder.metadata("title", title)?;
        builder.metadata("author", author)?;
        builder.metadata("lang", "en")?;
        let mut book=Self { builder, file };
        book.create_title_page(title, author)?;

        Ok(book)
    }
    pub fn new_with_path(title: &str,  author: &str,path:&str) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(format!("{}/{}",path,title))?;

        let file = File::create(format!("{}/{}.epub",path,title))?;
        let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

        // metadata
        builder.metadata("title", title)?;
        builder.metadata("author", author)?;
        builder.metadata("lang", "en")?;
        let mut book=Self { builder, file };
        book.create_title_page(title, author)?;

        Ok(book)
    }
    
    pub fn generate(self)->Result<(),Box<dyn std::error::Error>>{
        self.builder.generate(self.file)?;
        Ok(())
    }


    pub fn create_title_page(&mut self, title:&str, author:&str)->Result<(),Box<dyn std::error::Error>>{
        let title_page_html = format!(
            r#"
            <html xmlns="http://www.w3.org/1999/xhtml" lang="en">
            <head>
                <meta charset="UTF-8"/>
                <title>{}</title>
                <style>
                    body {{
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        height: 100vh;
                        text-align: center;
                        font-family: serif;
                        margin: 0;
                        background-color: #fff;
                    }}
                    h1 {{
                        font-size: 2em;
                        margin-bottom: 0.5em;
                    }}
                    h2 {{
                        font-size: 1.3em;
                        color: #555;
                    }}
                </style>
            </head>
            <body>
                <h1>{}</h1>
                <h2>{}</h2>
            </body>
            </html>
            "#,
            title,title,author
        );

        // Add the title page and mark it as the cover
        self.builder.add_content(
            EpubContent::new("titlepage.xhtml", title_page_html.as_bytes())
                .title("Cover")
                .reftype(ReferenceType::Cover),
        )?;
        Ok(())
    }

    pub fn add_chapter(&mut self,chapter:&Chapter)->Result<(),Box<dyn std::error::Error>>{
        let filename = format!("chapter{}.xhtml", chapter.num + 1);

        let re = Regex::new(r"(?i)\b(ch\.?|chapter)\s*[:\-]?\s*\d+").unwrap();

        let mut normalized=chapter.title.clone();
        if re.is_match(&chapter.title) {
            normalized=re.replace(&chapter.title, format!("Chapter {}", chapter.num)).to_string();
        }
        println!("{}",normalized);
        let chapter_content = format!(
            r#"
            <html xmlns="http://www.w3.org/1999/xhtml" lang="en">
            <head>
                <meta charset="UTF-8"/>
                <title>{title}</title>
                <style>
                    h1 {{
                        text-align: center;
                        font-size: 1.45em;   /* smaller, elegant size */
                        margin-top: 1em;
                        margin-bottom: 1em;
                    }}
                    body {{
                        font-family: serif;
                        line-height: 1.5;
                        margin: 1.5em;
                    }}
                </style>
            </head>
            <body>
                <h1>{title}</h1>
                {body}
            </body>
            </html>
            "#,
            title = normalized,
            body = chapter.html
        );
        self.builder.add_content(
            EpubContent::new(&filename, chapter_content.as_bytes())
                .title(&chapter.title)
                .reftype(ReferenceType::Text),
        )?;
        Ok(())
    }

}