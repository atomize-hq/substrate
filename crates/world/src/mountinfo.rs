use anyhow::{Context, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayBackingDirs {
    pub lowerdirs: Vec<String>,
    pub upperdir: Option<String>,
    pub workdir: Option<String>,
}

impl OverlayBackingDirs {
    pub fn is_empty(&self) -> bool {
        self.lowerdirs.is_empty() && self.upperdir.is_none() && self.workdir.is_none()
    }
}

/// Parse `/proc/self/mountinfo` and, if `mount_point` is an `overlay` mount, return its backing dirs.
pub fn overlay_backing_dirs_for_mount_point(
    mount_point: &str,
) -> Result<Option<OverlayBackingDirs>> {
    let raw = std::fs::read_to_string("/proc/self/mountinfo")
        .context("failed to read /proc/self/mountinfo")?;
    Ok(overlay_backing_dirs_for_mount_point_from_str(
        &raw,
        mount_point,
    ))
}

fn overlay_backing_dirs_for_mount_point_from_str(
    mountinfo: &str,
    mount_point: &str,
) -> Option<OverlayBackingDirs> {
    let mut best_match: Option<(u64, &str)> = None;

    for line in mountinfo.lines() {
        let Some((pre, post)) = line.split_once(" - ") else {
            continue;
        };

        let mut pre_fields = pre.split_whitespace();
        let mount_id_raw = pre_fields.next()?;
        let mount_id = match mount_id_raw.parse::<u64>() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let _parent_id = pre_fields.next()?;
        let _dev = pre_fields.next()?;
        let _root = pre_fields.next()?;
        let mp = pre_fields.next()?;

        let mp = decode_mountinfo_path(mp);
        if mp != mount_point {
            continue;
        }

        match best_match {
            Some((best_mount_id, _)) if mount_id <= best_mount_id => {}
            _ => best_match = Some((mount_id, post)),
        }
    }

    let (_mount_id, post) = best_match?;

    let mut post_fields = post.split_whitespace();
    let fstype = post_fields.next().unwrap_or("");
    let _mount_source = post_fields.next().unwrap_or("");
    let super_opts = post_fields.next().unwrap_or("");

    if fstype != "overlay" {
        return None;
    }

    let mut lowerdirs = Vec::new();
    let mut upperdir = None;
    let mut workdir = None;

    for opt in super_opts.split(',') {
        if let Some(raw) = opt.strip_prefix("lowerdir=") {
            for p in raw.split(':').filter(|p| !p.trim().is_empty()) {
                lowerdirs.push(decode_mountinfo_path(p.trim()));
            }
            continue;
        }

        if let Some(raw) = opt.strip_prefix("upperdir=") {
            let t = raw.trim();
            if !t.is_empty() {
                upperdir = Some(decode_mountinfo_path(t));
            }
            continue;
        }

        if let Some(raw) = opt.strip_prefix("workdir=") {
            let t = raw.trim();
            if !t.is_empty() {
                workdir = Some(decode_mountinfo_path(t));
            }
            continue;
        }
    }

    Some(OverlayBackingDirs {
        lowerdirs,
        upperdir,
        workdir,
    })
}

fn decode_mountinfo_path(input: &str) -> String {
    // mountinfo escaping uses octal escapes: \040 for space, \011 tab, \012 newline, \134 backslash.
    // Ref: `man proc` (mountinfo).
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        let Some(a) = chars.peek().copied() else {
            out.push('\\');
            break;
        };
        if !a.is_ascii_digit() {
            out.push('\\');
            continue;
        }

        let a = chars.next().unwrap();
        let b = chars.next().unwrap_or('\0');
        let d = chars.next().unwrap_or('\0');
        if !(b.is_ascii_digit() && d.is_ascii_digit()) {
            out.push('\\');
            out.push(a);
            if b != '\0' {
                out.push(b);
            }
            if d != '\0' {
                out.push(d);
            }
            continue;
        }

        let oct = [a, b, d].into_iter().collect::<String>().to_string();
        if let Ok(val) = u8::from_str_radix(&oct, 8) {
            out.push(val as char);
        } else {
            out.push('\\');
            out.push_str(&oct);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_mountinfo_path_unescapes_space() {
        assert_eq!(
            decode_mountinfo_path("/tmp/hello\\040world"),
            "/tmp/hello world"
        );
    }

    #[test]
    fn overlay_backing_dirs_parses_overlay_superopts() {
        let mi = "\
43 33 0:44 / /tmp/tmp.abc rw,relatime - overlay overlay rw,lowerdir=/lower,upperdir=/upper,workdir=/work,index=off\n\
";
        let dirs = overlay_backing_dirs_for_mount_point_from_str(mi, "/tmp/tmp.abc").unwrap();
        assert_eq!(
            dirs,
            OverlayBackingDirs {
                lowerdirs: vec!["/lower".to_string()],
                upperdir: Some("/upper".to_string()),
                workdir: Some("/work".to_string()),
            }
        );
    }

    #[test]
    fn overlay_backing_dirs_splits_multiple_lowerdirs() {
        let mi = "\
43 33 0:44 / /mnt/project rw,relatime - overlay overlay rw,lowerdir=/l1:/l2:/l3,upperdir=/u,workdir=/w\n\
";
        let dirs = overlay_backing_dirs_for_mount_point_from_str(mi, "/mnt/project").unwrap();
        assert_eq!(dirs.lowerdirs, vec!["/l1", "/l2", "/l3"]);
        assert_eq!(dirs.upperdir.as_deref(), Some("/u"));
        assert_eq!(dirs.workdir.as_deref(), Some("/w"));
    }

    #[test]
    fn overlay_backing_dirs_decodes_escaped_backing_dirs() {
        let mi = "\
43 33 0:44 / /mnt/project rw,relatime - overlay overlay rw,lowerdir=/l1\\040space:/l2\\134slash,upperdir=/u\\040space,workdir=/w\\040space\n\
";
        let dirs = overlay_backing_dirs_for_mount_point_from_str(mi, "/mnt/project").unwrap();
        assert_eq!(dirs.lowerdirs, vec!["/l1 space", "/l2\\slash"]);
        assert_eq!(dirs.upperdir.as_deref(), Some("/u space"));
        assert_eq!(dirs.workdir.as_deref(), Some("/w space"));
    }

    #[test]
    fn overlay_backing_dirs_matches_decoded_mountpoint() {
        let mi = "\
43 33 0:44 / /tmp/hello\\040world rw,relatime - overlay overlay rw,lowerdir=/l,upperdir=/u,workdir=/w\n\
";
        let dirs = overlay_backing_dirs_for_mount_point_from_str(mi, "/tmp/hello world").unwrap();
        assert_eq!(dirs.upperdir.as_deref(), Some("/u"));
    }

    #[test]
    fn overlay_backing_dirs_selects_highest_mount_id_for_duplicate_mountpoint() {
        let mi = "\
43 33 0:44 / /mnt/project rw,relatime - overlay overlay rw,lowerdir=/l1,upperdir=/u43,workdir=/w43\n\
99 33 0:44 / /mnt/project rw,relatime - overlay overlay rw,lowerdir=/l2,upperdir=/u99,workdir=/w99\n\
";
        let dirs = overlay_backing_dirs_for_mount_point_from_str(mi, "/mnt/project").unwrap();
        assert_eq!(dirs.lowerdirs, vec!["/l2"]);
        assert_eq!(dirs.upperdir.as_deref(), Some("/u99"));
        assert_eq!(dirs.workdir.as_deref(), Some("/w99"));
    }
}
