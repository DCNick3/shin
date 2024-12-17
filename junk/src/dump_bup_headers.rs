pub fn main(root_path: String) {
    walkdir::WalkDir::new(&root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "bup"))
        .for_each(|e| {
            let path = e.path();
            let file = std::fs::File::open(path).unwrap();
            let mut reader = std::io::BufReader::new(file);

            // poor man's csv
            let path = path.strip_prefix(&root_path).unwrap(); // get path relative to the root
            let prefix = format!("\"{}\", ", path.display());

            // println!(
            //     "{}",
            //     shin_core::format::bustup::dump_header(&mut reader, &prefix).unwrap()
            // );
            print!(
                "{}",
                shin_core::format::bustup::dump_expression_descriptors(&mut reader, &prefix)
                    .unwrap()
            );
        });
}
