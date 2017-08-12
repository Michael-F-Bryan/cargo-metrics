extern crate skeptic;


fn main() {
    let mut mdbook_files = skeptic::markdown_files_of_directory("guide/");
    mdbook_files.push("README.md".to_string());

    skeptic::generate_doc_tests(&mdbook_files);
}