//! Performance benchmarks for dei
//!
//! These benchmarks measure real-world performance across different scenarios.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use dei_ast::{AstBuilder, ParallelTraverser};
use dei_core::thresholds::Thresholds;
use dei_e2e::{FixtureManager, TestHarness};
use dei_languages::MultiLanguageParser;
use std::path::PathBuf;

fn bench_single_file_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_file");
    
    let fixture = FixtureManager::new().expect("Failed to create fixture");
    let healthy_path = fixture.copy_fixture("rust").expect("Failed to copy fixture");
    
    group.bench_function("healthy_rust", |b| {
        b.iter(|| {
            let harness = TestHarness::new().expect("Failed to create harness");
            black_box(harness.analyze_path(healthy_path.join("healthy.rs")).unwrap())
        });
    });
    
    group.bench_function("god_class_rust", |b| {
        b.iter(|| {
            let harness = TestHarness::new().expect("Failed to create harness");
            black_box(harness.analyze_path(healthy_path.join("god_class.rs")).unwrap())
        });
    });
    
    group.finish();
}

fn bench_directory_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory");
    
    for num_files in [10, 50, 100].iter() {
        let fixture = FixtureManager::new().expect("Failed to create fixture");
        
        // Create multiple files
        for i in 0..*num_files {
            fixture.create_file(
                &format!("bench{}/file{}.rs", num_files, i),
                include_str!("../fixtures/rust/healthy.rs"),
            ).expect("Failed to create file");
        }
        
        group.throughput(Throughput::Elements(*num_files as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_files),
            num_files,
            |b, _| {
                b.iter(|| {
                    let harness = TestHarness::new().expect("Failed to create harness");
                    let path = fixture.path().join(format!("bench{}", num_files));
                    black_box(harness.analyze_path(path).unwrap())
                });
            },
        );
    }
    
    group.finish();
}

fn bench_ast_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast_building");
    
    let fixture = FixtureManager::new().expect("Failed to create fixture");
    
    // Create nested directory structure
    for i in 0..20 {
        fixture.create_file(
            &format!("project/src/module{}/lib.rs", i),
            "pub struct Test;",
        ).expect("Failed to create file");
    }
    
    let project_path = fixture.path().join("project");
    
    group.bench_function("build_filesystem_ast", |b| {
        b.iter(|| {
            let builder = AstBuilder::new();
            black_box(builder.build(&project_path).unwrap())
        });
    });
    
    group.finish();
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    
    let fixture = FixtureManager::new().expect("Failed to create fixture");
    let path = fixture.copy_fixture("rust").expect("Failed to copy fixture");
    
    group.bench_function("parse_healthy_rust", |b| {
        let parser = MultiLanguageParser::new().expect("Failed to create parser");
        let file_path = path.join("healthy.rs");
        
        b.iter(|| {
            black_box(parser.parse_file(&file_path).unwrap())
        });
    });
    
    group.bench_function("parse_god_class_rust", |b| {
        let parser = MultiLanguageParser::new().expect("Failed to create parser");
        let file_path = path.join("god_class.rs");
        
        b.iter(|| {
            black_box(parser.parse_file(&file_path).unwrap())
        });
    });
    
    group.finish();
}

fn bench_parallel_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_traversal");
    
    let fixture = FixtureManager::new().expect("Failed to create fixture");
    
    // Create many files for parallel processing
    for i in 0..100 {
        fixture.create_file(
            &format!("parallel/file{}.rs", i),
            include_str!("../fixtures/rust/healthy.rs"),
        ).expect("Failed to create file");
    }
    
    let project_path = fixture.path().join("parallel");
    
    group.bench_function("traverse_100_files", |b| {
        b.iter(|| {
            let builder = AstBuilder::new();
            let root_id = builder.build(&project_path).unwrap();
            
            let parser = MultiLanguageParser::new().unwrap();
            let traverser = ParallelTraverser::new(parser, builder.arena().clone());
            let thresholds = Thresholds::default();
            
            traverser.traverse_and_analyze(root_id, &thresholds).unwrap();
            black_box(traverser.all_results())
        });
    });
    
    group.finish();
}

fn bench_large_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_file");
    
    let fixture = FixtureManager::new().expect("Failed to create fixture");
    
    // Generate a large file
    let mut large_code = String::from("pub struct Large {\n");
    for i in 0..500 {
        large_code.push_str(&format!("    field{}: u32,\n", i));
    }
    large_code.push_str("}\n\nimpl Large {\n");
    for i in 0..500 {
        large_code.push_str(&format!(
            "    pub fn method{}(&self) -> u32 {{ self.field{} }}\n",
            i, i
        ));
    }
    large_code.push_str("}\n");
    
    fixture.create_file("large.rs", &large_code).expect("Failed to create file");
    
    group.bench_function("analyze_1000_line_file", |b| {
        b.iter(|| {
            let harness = TestHarness::new().expect("Failed to create harness");
            black_box(harness.analyze_path(fixture.path().join("large.rs")).unwrap())
        });
    });
    
    group.finish();
}

fn bench_multi_language(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_language");
    
    let fixture = FixtureManager::new().expect("Failed to create fixture");
    
    // Create mixed language files
    for i in 0..20 {
        fixture.create_file(
            &format!("mixed/file{}.rs", i),
            include_str!("../fixtures/rust/healthy.rs"),
        ).expect("Failed to create file");
        
        fixture.create_file(
            &format!("mixed/File{}.cs", i),
            include_str!("../fixtures/csharp/Healthy.cs"),
        ).expect("Failed to create file");
    }
    
    group.bench_function("analyze_mixed_40_files", |b| {
        b.iter(|| {
            let harness = TestHarness::new().expect("Failed to create harness");
            black_box(harness.analyze_path(fixture.path().join("mixed")).unwrap())
        });
    });
    
    group.finish();
}

fn bench_threshold_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("thresholds");
    
    let fixture = FixtureManager::new().expect("Failed to create fixture");
    let path = fixture.copy_fixture("rust").expect("Failed to copy fixture");
    
    group.bench_function("strict_thresholds", |b| {
        use dei_core::thresholds::{Complexity, Lines, MethodCount};
        use dei_e2e::ThresholdBuilder;
        
        let thresholds = ThresholdBuilder::new()
            .max_class_lines(50)
            .max_methods(5)
            .max_class_complexity(10)
            .build();
        
        b.iter(|| {
            let harness = TestHarness::new().unwrap().with_thresholds(thresholds.clone());
            black_box(harness.analyze_path(path.join("god_class.rs")).unwrap())
        });
    });
    
    group.bench_function("lenient_thresholds", |b| {
        use dei_e2e::ThresholdBuilder;
        
        let thresholds = ThresholdBuilder::new()
            .max_class_lines(10000)
            .max_methods(1000)
            .max_class_complexity(1000)
            .build();
        
        b.iter(|| {
            let harness = TestHarness::new().unwrap().with_thresholds(thresholds.clone());
            black_box(harness.analyze_path(path.join("god_class.rs")).unwrap())
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_file_analysis,
    bench_directory_analysis,
    bench_ast_building,
    bench_parsing,
    bench_parallel_traversal,
    bench_large_file,
    bench_multi_language,
    bench_threshold_variations,
);

criterion_main!(benches);

