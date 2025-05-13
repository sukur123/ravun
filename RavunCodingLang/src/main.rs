mod lexer;
mod parser;
mod semantics;
mod ir;
mod optimizer;
mod codegen;
mod utils;

use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::semantics::analyzer::SemanticAnalyzer;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Kullanım: {} <dosya.rv>", args[0]);
        return Ok(());
    }
    
    let file_path = &args[1];
    let source = read_source_file(file_path)?;
    
    println!("Ravun Derleyicisi");
    println!("Dosya: {}", file_path);
    
    if let Err(err) = compile(&source) {
        eprintln!("Derleme hatası: {}", err);
        std::process::exit(1);
    }
    
    println!("Derleme başarılı!");
    
    Ok(())
}

fn read_source_file(file_path: &str) -> io::Result<String> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Dosya bulunamadı: {}", file_path),
        ));
    }
    
    let extension = path.extension().and_then(|ext| ext.to_str());
    
    if extension != Some("rv") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Geçersiz dosya uzantısı: {}, .rv bekleniyor", file_path),
        ));
    }
    
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    Ok(content)
}

fn compile(source: &str) -> Result<(), String> {
    println!("Lexical analiz yapılıyor...");
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    
    println!("Tokenlar:");
    for token in &tokens {
        println!("{:?} - '{}'", token.token_type, token.lexeme);
    }
    
    println!("Parsing işlemi yapılıyor...");
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(errors) => {
            for error in errors {
                eprintln!("Parser hatası: {}", error);
            }
            return Err("Parsing hatası".to_string());
        }
    };
    
    println!("AST:\n{:#?}", ast);
    
    println!("Semantik analiz yapılıyor...");
    let mut analyzer = SemanticAnalyzer::new();
    let semantic_errors = analyzer.analyze(&ast);
    
    if !semantic_errors.is_empty() {
        println!("Semantik analiz hataları:");
        for error in semantic_errors {
            eprintln!("Semantik hata: {}", error);
        }
        return Err("Semantik analiz hatası".to_string());
    }
    
    println!("Semantik analiz başarılı.");
    
    println!("IR kodu oluşturuluyor...");
    
    println!("Optimizasyon yapılıyor...");
    
    println!("Kod üretimi yapılıyor...");
    
    Ok(())
}

mod parser;
mod semantics;

use parser::parse;
use semantics::analyzer::SemanticAnalyzer;

fn main() {
    let source = std::fs::read_to_string("hello_world.rv").expect("Dosya okunamadı");

    let ast = parse(&source).expect("Parse hatası!");

    let mut analyzer = SemanticAnalyzer::new();
    let errors = analyzer.analyze(&ast);

    if errors.is_empty() {
        println!("✅ Semantic analysis passed with no errors.");
    } else {
        println!("❌ Semantic errors found:");
        for err in errors {
            println!("- {}", err.message);
        }
    }
}

