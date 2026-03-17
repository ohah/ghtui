use ghtui_core::types::{DiffFile, DiffFileStatus, DiffHunk, DiffLine, DiffLineKind};

pub fn parse_diff(raw: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<DiffFileBuilder> = None;
    let mut current_hunk: Option<HunkBuilder> = None;

    for line in raw.lines() {
        if line.starts_with("diff --git") {
            // Finish previous file
            if let Some(mut file) = current_file.take() {
                if let Some(hunk) = current_hunk.take() {
                    file.hunks.push(hunk.build());
                }
                files.push(file.build());
            }
            current_file = Some(DiffFileBuilder::from_header(line));
        } else if line.starts_with("--- ") {
            if let Some(ref mut file) = current_file {
                file.old_name = line.strip_prefix("--- a/").map(String::from);
            }
        } else if line.starts_with("+++ ") {
            if let Some(ref mut file) = current_file {
                file.new_name = line.strip_prefix("+++ b/").map(String::from);
            }
        } else if line.starts_with("new file") {
            if let Some(ref mut file) = current_file {
                file.status = DiffFileStatus::Added;
            }
        } else if line.starts_with("deleted file") {
            if let Some(ref mut file) = current_file {
                file.status = DiffFileStatus::Removed;
            }
        } else if line.starts_with("rename from") || line.starts_with("rename to") {
            if let Some(ref mut file) = current_file {
                file.status = DiffFileStatus::Renamed;
            }
        } else if line.starts_with("@@") {
            // New hunk
            if let Some(ref mut file) = current_file {
                if let Some(hunk) = current_hunk.take() {
                    file.hunks.push(hunk.build());
                }
            }
            current_hunk = Some(HunkBuilder::from_header(line));
        } else if current_hunk.is_some() {
            let hunk = current_hunk.as_mut().unwrap();
            if let Some(content) = line.strip_prefix('+') {
                hunk.add_line(DiffLineKind::Add, content);
            } else if let Some(content) = line.strip_prefix('-') {
                hunk.add_line(DiffLineKind::Remove, content);
            } else if let Some(content) = line.strip_prefix(' ') {
                hunk.add_line(DiffLineKind::Context, content);
            } else if line == "\\ No newline at end of file" {
                // skip
            } else {
                hunk.add_line(DiffLineKind::Context, line);
            }
        }
    }

    // Finish last file
    if let Some(mut file) = current_file.take() {
        if let Some(hunk) = current_hunk.take() {
            file.hunks.push(hunk.build());
        }
        files.push(file.build());
    }

    files
}

struct DiffFileBuilder {
    filename: String,
    old_name: Option<String>,
    new_name: Option<String>,
    status: DiffFileStatus,
    hunks: Vec<DiffHunk>,
}

impl DiffFileBuilder {
    fn from_header(line: &str) -> Self {
        // "diff --git a/path b/path"
        let filename = line.split(" b/").nth(1).unwrap_or("unknown").to_string();

        Self {
            filename,
            old_name: None,
            new_name: None,
            status: DiffFileStatus::Modified,
            hunks: Vec::new(),
        }
    }

    fn build(self) -> DiffFile {
        let mut additions = 0u32;
        let mut deletions = 0u32;

        for hunk in &self.hunks {
            for line in &hunk.lines {
                match line.kind {
                    DiffLineKind::Add => additions += 1,
                    DiffLineKind::Remove => deletions += 1,
                    _ => {}
                }
            }
        }

        let filename = self.new_name.unwrap_or(self.filename);

        DiffFile {
            filename,
            status: self.status,
            additions,
            deletions,
            hunks: self.hunks,
        }
    }
}

struct HunkBuilder {
    header: String,
    old_start: u32,
    new_start: u32,
    old_line: u32,
    new_line: u32,
    lines: Vec<DiffLine>,
}

impl HunkBuilder {
    fn from_header(line: &str) -> Self {
        let (old_start, new_start) = parse_hunk_header(line);
        Self {
            header: line.to_string(),
            old_start,
            new_start,
            old_line: old_start,
            new_line: new_start,
            lines: Vec::new(),
        }
    }

    fn add_line(&mut self, kind: DiffLineKind, content: &str) {
        let (old_line, new_line) = match kind {
            DiffLineKind::Context => {
                let ol = self.old_line;
                let nl = self.new_line;
                self.old_line += 1;
                self.new_line += 1;
                (Some(ol), Some(nl))
            }
            DiffLineKind::Add => {
                let nl = self.new_line;
                self.new_line += 1;
                (None, Some(nl))
            }
            DiffLineKind::Remove => {
                let ol = self.old_line;
                self.old_line += 1;
                (Some(ol), None)
            }
            DiffLineKind::Header => (None, None),
        };

        self.lines.push(DiffLine {
            kind,
            content: content.to_string(),
            old_line,
            new_line,
        });
    }

    fn build(self) -> DiffHunk {
        DiffHunk {
            header: self.header,
            old_start: self.old_start,
            new_start: self.new_start,
            lines: self.lines,
        }
    }
}

fn parse_hunk_header(header: &str) -> (u32, u32) {
    // @@ -old_start,old_count +new_start,new_count @@
    let parts: Vec<&str> = header.split(' ').collect();
    let old_start = parts
        .get(1)
        .and_then(|s| s.strip_prefix('-'))
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let new_start = parts
        .get(2)
        .and_then(|s| s.strip_prefix('+'))
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    (old_start, new_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_diff() {
        let diff = r#"diff --git a/hello.txt b/hello.txt
--- a/hello.txt
+++ b/hello.txt
@@ -1,3 +1,4 @@
 line1
-line2
+line2_modified
+line3_new
 line4"#;

        let files = parse_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "hello.txt");
        assert_eq!(files[0].status, DiffFileStatus::Modified);
        assert_eq!(files[0].additions, 2);
        assert_eq!(files[0].deletions, 1);
        assert_eq!(files[0].hunks.len(), 1);
    }

    #[test]
    fn test_parse_new_file_diff() {
        let diff = r#"diff --git a/new.txt b/new.txt
new file mode 100644
--- /dev/null
+++ b/new.txt
@@ -0,0 +1,2 @@
+hello
+world"#;

        let files = parse_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, DiffFileStatus::Added);
        assert_eq!(files[0].additions, 2);
    }

    #[test]
    fn test_parse_hunk_header() {
        assert_eq!(parse_hunk_header("@@ -1,3 +1,4 @@"), (1, 1));
        assert_eq!(parse_hunk_header("@@ -10,5 +20,8 @@ fn foo"), (10, 20));
    }
}
