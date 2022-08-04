use wasm_opt::base::*;

use std::fs::{self, File};
use std::io::BufWriter;
use std::io::Write;
use tempfile::Builder;

static WAT_FILE: &[u8] = include_bytes!("hello_world.wat");
static WASM_FILE: &[u8] = include_bytes!("hello_world.wasm");

#[test]
fn read_write_text_works() -> anyhow::Result<()> {
    let temp_dir = Builder::new().prefix("wasm_opt_tests").tempdir()?;
    let path = temp_dir.path().join("hello_world.wat");

    let temp_file = File::create(&path)?;
    let mut buf_writer = BufWriter::new(&temp_file);
    buf_writer.write_all(WAT_FILE)?;

    let mut m = Module::new();
    let mut reader = ModuleReader::new();
    reader.read_text(&path, &mut m)?;

    let mut writer = ModuleWriter::new();
    let new_file = temp_dir.path().join("hello_world_by_module_writer.wat");
    writer.write_text(&mut m, &new_file)?;

    let mut another_m = Module::new();
    let mut another_reader = ModuleReader::new();
    another_reader.read_text(&new_file, &mut another_m)?;

    let mut another_writer = ModuleWriter::new();
    let another_new_file = temp_dir
        .path()
        .join("hello_world_by_another_module_writer.wat");
    another_writer.write_text(&mut another_m, &another_new_file)?;

    let new_file_reader = fs::read(&new_file)?;
    let another_new_file_reader = fs::read(&another_new_file)?;

    assert_eq!(new_file_reader, another_new_file_reader);

    Ok(())
}

#[test]
fn read_write_binary_works() -> anyhow::Result<()> {
    let temp_dir = Builder::new().prefix("wasm_opt_tests").tempdir()?;
    let path = temp_dir.path().join("hello_world.wasm");

    let temp_file = File::create(&path)?;
    let mut buf_writer = BufWriter::new(&temp_file);
    buf_writer.write_all(WASM_FILE)?;

    let mut m = Module::new();
    let mut reader = ModuleReader::new();
    reader.read_binary(&path, &mut m, None)?;

    let mut writer = ModuleWriter::new();
    let new_file = temp_dir.path().join("hello_world_by_module_writer.wasm");
    writer.write_binary(&mut m, &new_file)?;

    let mut another_m = Module::new();
    let mut another_reader = ModuleReader::new();
    another_reader.read_binary(&new_file, &mut another_m, None)?;
    let mut another_writer = ModuleWriter::new();
    let another_new_file = temp_dir
        .path()
        .join("hello_world_by_another_module_writer.wasm");
    another_writer.write_binary(&mut another_m, &another_new_file)?;

    let new_file_reader = fs::read(&new_file)?;
    let another_new_file_reader = fs::read(&another_new_file)?;

    assert_eq!(new_file_reader, another_new_file_reader);

    Ok(())
}

#[test]
fn pass_runner_works() -> anyhow::Result<()> {
    let temp_dir = Builder::new().prefix("wasm_opt_tests").tempdir()?;
    let path = temp_dir.path().join("hello_world.wasm");

    let temp_file = File::create(&path)?;
    let mut buf_writer = BufWriter::new(&temp_file);
    buf_writer.write_all(WASM_FILE)?;
    // Module without optimization
    let mut m = Module::new();
    let mut reader = ModuleReader::new();
    reader.read_binary(&path, &mut m, None)?;

    let mut writer = ModuleWriter::new();
    let new_file = temp_dir.path().join("hello_world_by_module_writer.wasm");
    writer.write_binary(&mut m, &new_file)?;

    // Module with optimization
    let mut another_m = Module::new();
    let mut another_reader = ModuleReader::new();
    another_reader.read_binary(&new_file, &mut another_m, None)?;

    let mut pass_runner = PassRunner::new(&mut another_m);
    pass_runner.add_default_optimization_passes();
    pass_runner.run();
    drop(pass_runner);

    let mut another_writer = ModuleWriter::new();
    let another_new_file = temp_dir
        .path()
        .join("hello_world_by_another_module_writer.wasm");
    another_writer.write_binary(&mut another_m, &another_new_file)?;

    let new_file_reader = fs::read(&new_file)?;
    let another_new_file_reader = fs::read(&another_new_file)?;

    assert!(new_file_reader.len() > another_new_file_reader.len());

    Ok(())
}

#[test]
fn pass_options_works() -> anyhow::Result<()> {
    let temp_dir = Builder::new().prefix("wasm_opt_tests").tempdir()?;
    let path = temp_dir.path().join("hello_world.wasm");

    let temp_file = File::create(&path)?;
    let mut buf_writer = BufWriter::new(&temp_file);
    buf_writer.write_all(WASM_FILE)?;

    // Module without optimization
    let mut m = Module::new();
    let mut reader = ModuleReader::new();
    reader.read_binary(&path, &mut m, None)?;

    let mut writer = ModuleWriter::new();
    let new_file = temp_dir.path().join("hello_world_by_module_writer.wasm");
    writer.write_binary(&mut m, &new_file)?;

    // Module with optimization: default
    let mut m_0 = Module::new();
    let mut reader_0 = ModuleReader::new();
    reader_0.read_binary(&new_file, &mut m_0, None)?;

    let mut pass_options = PassOptions::new();
    pass_options.set_optimize_level(2);
    pass_options.set_shrink_level(1);

    let mut pass_runner = PassRunner::new_with_options(&mut m_0, pass_options);
    pass_runner.add_default_optimization_passes();
    pass_runner.run();
    drop(pass_runner);

    let mut writer_0 = ModuleWriter::new();
    let file_0 = temp_dir.path().join("hello_world_by_module_writer_0.wasm");
    writer_0.write_binary(&mut m_0, &file_0)?;

    let new_file_reader = fs::read(&new_file)?;
    let file_reader_0 = fs::read(&file_0)?;

    println!("new_file: {}", new_file_reader.len());
    println!("file_0: {}", file_reader_0.len());

    assert!(new_file_reader.len() > file_reader_0.len());

    // Module with optimization: more optimized settings
    let mut m_1 = Module::new();
    let mut reader_1 = ModuleReader::new();
    reader_1.read_binary(&new_file, &mut m_1, None)?;

    let mut pass_options = PassOptions::new();
    pass_options.set_optimize_level(5);
    pass_options.set_shrink_level(5);

    let mut pass_runner = PassRunner::new_with_options(&mut m_1, pass_options);
    pass_runner.add_default_optimization_passes();
    pass_runner.run();
    drop(pass_runner);

    let mut writer_1 = ModuleWriter::new();
    let file_1 = temp_dir.path().join("hello_world_by_module_writer_1.wasm");
    writer_1.write_binary(&mut m_1, &file_1)?;

    let file_reader_1 = fs::read(&file_1)?;
    println!("file_1: {}", file_reader_1.len());

    assert!(file_reader_0.len() > file_reader_1.len());

    // Module with optimization: ridiculous settings
    let mut m_2 = Module::new();
    let mut reader_2 = ModuleReader::new();
    reader_2.read_binary(&new_file, &mut m_2, None)?;

    let mut pass_options = PassOptions::new();

    pass_options.set_optimize_level(2_000_000_000);
    pass_options.set_shrink_level(2_000_000_000);

    let mut pass_runner = PassRunner::new_with_options(&mut m_2, pass_options);
    pass_runner.add_default_optimization_passes();
    pass_runner.run();
    drop(pass_runner);

    let mut writer_2 = ModuleWriter::new();
    let file_2 = temp_dir.path().join("hello_world_by_module_writer_2.wasm");
    writer_2.write_binary(&mut m_2, &file_2)?;

    let file_reader_2 = fs::read(&file_2)?;
    println!("file_2: {}", file_reader_2.len());

    assert!(file_reader_1.len() >= file_reader_2.len());

    Ok(())
}