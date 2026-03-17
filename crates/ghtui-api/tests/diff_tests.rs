use ghtui_api::diff::parse_diff;
use ghtui_core::types::{DiffFileStatus, DiffLineKind};

#[test]
fn test_parse_empty_diff() {
    let files = parse_diff("");
    assert!(files.is_empty());
}

#[test]
fn test_parse_single_file_diff() {
    let diff = r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,5 +1,6 @@
 fn main() {
-    println!("Hello");
+    println!("Hello, world!");
+    println!("Goodbye");
     let x = 1;
     let y = 2;
 }"#;

    let files = parse_diff(diff);
    assert_eq!(files.len(), 1);

    let file = &files[0];
    assert_eq!(file.filename, "src/main.rs");
    assert_eq!(file.status, DiffFileStatus::Modified);
    assert_eq!(file.additions, 2);
    assert_eq!(file.deletions, 1);
    assert_eq!(file.hunks.len(), 1);

    let hunk = &file.hunks[0];
    assert_eq!(hunk.old_start, 1);
    assert_eq!(hunk.new_start, 1);

    // Check line types
    let line_kinds: Vec<DiffLineKind> = hunk.lines.iter().map(|l| l.kind).collect();
    assert_eq!(
        line_kinds,
        vec![
            DiffLineKind::Context,
            DiffLineKind::Remove,
            DiffLineKind::Add,
            DiffLineKind::Add,
            DiffLineKind::Context,
            DiffLineKind::Context,
            DiffLineKind::Context,
        ]
    );
}

#[test]
fn test_parse_new_file() {
    let diff = r#"diff --git a/new_file.txt b/new_file.txt
new file mode 100644
--- /dev/null
+++ b/new_file.txt
@@ -0,0 +1,3 @@
+line 1
+line 2
+line 3"#;

    let files = parse_diff(diff);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].status, DiffFileStatus::Added);
    assert_eq!(files[0].additions, 3);
    assert_eq!(files[0].deletions, 0);
}

#[test]
fn test_parse_deleted_file() {
    let diff = r#"diff --git a/old_file.txt b/old_file.txt
deleted file mode 100644
--- a/old_file.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-line 1
-line 2"#;

    let files = parse_diff(diff);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].status, DiffFileStatus::Removed);
    assert_eq!(files[0].additions, 0);
    assert_eq!(files[0].deletions, 2);
}

#[test]
fn test_parse_multiple_files() {
    let diff = r#"diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,1 +1,2 @@
 existing
+new line
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1,2 +1,1 @@
 keep
-remove"#;

    let files = parse_diff(diff);
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].filename, "file1.txt");
    assert_eq!(files[0].additions, 1);
    assert_eq!(files[1].filename, "file2.txt");
    assert_eq!(files[1].deletions, 1);
}

#[test]
fn test_parse_renamed_file() {
    let diff = r#"diff --git a/old_name.txt b/new_name.txt
rename from old_name.txt
rename to new_name.txt
--- a/old_name.txt
+++ b/new_name.txt
@@ -1,1 +1,1 @@
-old content
+new content"#;

    let files = parse_diff(diff);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].status, DiffFileStatus::Renamed);
    assert_eq!(files[0].filename, "new_name.txt");
}

#[test]
fn test_line_numbers() {
    let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -10,4 +10,5 @@
 context at 10
-removed at 11
+added at 11
+added at 12
 context at 12/13
 context at 13/14"#;

    let files = parse_diff(diff);
    let hunk = &files[0].hunks[0];

    // Context line at 10
    assert_eq!(hunk.lines[0].old_line, Some(10));
    assert_eq!(hunk.lines[0].new_line, Some(10));

    // Removed line
    assert_eq!(hunk.lines[1].old_line, Some(11));
    assert_eq!(hunk.lines[1].new_line, None);

    // Added lines
    assert_eq!(hunk.lines[2].old_line, None);
    assert_eq!(hunk.lines[2].new_line, Some(11));
    assert_eq!(hunk.lines[3].old_line, None);
    assert_eq!(hunk.lines[3].new_line, Some(12));
}

#[test]
fn test_multiple_hunks() {
    let diff = r#"diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,3 +1,3 @@
 a
-b
+B
 c
@@ -10,3 +10,3 @@
 x
-y
+Y
 z"#;

    let files = parse_diff(diff);
    assert_eq!(files[0].hunks.len(), 2);
    assert_eq!(files[0].hunks[0].old_start, 1);
    assert_eq!(files[0].hunks[1].old_start, 10);
}
