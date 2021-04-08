use std::fs::File;

use tempfile::tempdir;

use crate::importer::{clean_empty_dirs, move_and_sort, Config, Language};

#[test]
fn integration_test() {
    let dir = tempdir().unwrap();
    let f1 = dir.path().join("1959-03-12.txt");
    File::create(&f1).unwrap();

    let mut config = Config {
        input: dir.path().to_path_buf(),
        output: dir.path().to_path_buf(),
        verbose: false,
        name: true,
        meta: true,
        clean: true,
        min_year: 1950,
        year: false,
        month: Language::None,
        flat: false,
    };

    move_and_sort(&config);
    assert!(dir.path().join("1959/03/1959-03-12.txt").exists());

    config.year = true;
    move_and_sort(&config);
    assert!(dir.path().join("1959/1959-03/1959-03-12.txt").exists());

    config.flat = true;
    move_and_sort(&config);
    assert!(dir.path().join("1959-03/1959-03-12.txt").exists());

    config.year = false;
    config.flat = false;
    config.month = Language::English;
    move_and_sort(&config);
    assert!(dir.path().join("1959/03 March/1959-03-12.txt").exists());

    config.month = Language::None;
    config.name = false;
    move_and_sort(&config);
    assert!(!dir.path().join("1959/03/1959-03-12.txt").exists());

    config.name = true;
    config.meta = false;
    move_and_sort(&config);
    assert!(dir.path().join("1959/03/1959-03-12.txt").exists());

    config.name = true;
    config.meta = true;
    move_and_sort(&config);
    assert!(dir.path().join("1959/03/1959-03-12.txt").exists());

    config.min_year = 2000;
    move_and_sort(&config);
    assert!(!dir.path().join("1959/03/1959-03-12.txt").exists());

    clean_empty_dirs(&config.input, config.verbose);
    assert_eq!(dir.path().read_dir().unwrap().count(), 1);
}
