// Sandbox policy integration tests

#[cfg(test)]
mod sandbox_tests {
    use flame_sandbox::{Capability, SandboxPolicy};
    use std::path::PathBuf;

    #[test]
    fn deny_all_denies_everything() {
        let policy = SandboxPolicy::deny_all();
        assert!(!policy.check_clock());
        assert!(!policy.check_random());
        assert!(!policy.check_proc_exit());
        assert!(!policy.check_read(&PathBuf::from("/tmp")));
        assert!(!policy.check_write(&PathBuf::from("/tmp")));
    }

    #[test]
    fn deny_all_with_clock_grant() {
        let policy = SandboxPolicy::deny_all().grant(Capability::Clock);
        assert!(policy.check_clock());
        assert!(!policy.check_random()); // still denied
    }

    #[test]
    fn allow_all_permits_everything() {
        let policy = SandboxPolicy::allow_all();
        assert!(policy.check_clock());
        assert!(policy.check_random());
        assert!(policy.check_read(&PathBuf::from("/etc/passwd")));
        assert!(policy.check_write(&PathBuf::from("/tmp/file")));
    }

    #[test]
    fn read_dir_grant_allows_subtree() {
        let policy = SandboxPolicy::deny_all()
            .grant(Capability::ReadDir(PathBuf::from("/var/data")));
        assert!(policy.check_read(&PathBuf::from("/var/data/file.txt")));
        assert!(policy.check_read(&PathBuf::from("/var/data/sub/dir/f")));
        assert!(!policy.check_read(&PathBuf::from("/var/other")));
        assert!(!policy.check_write(&PathBuf::from("/var/data/file.txt"))); // read-only grant
    }

    #[test]
    fn write_dir_grant_allows_read_and_write() {
        let policy = SandboxPolicy::deny_all()
            .grant(Capability::WriteDir(PathBuf::from("/tmp/out")));
        assert!(policy.check_read(&PathBuf::from("/tmp/out/result")));
        assert!(policy.check_write(&PathBuf::from("/tmp/out/result")));
    }
}
