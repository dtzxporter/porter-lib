use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

/// A trait that adds methods to path types.
pub trait PathExt {
    /// Construct a relative path from a provided base directory path to this path.
    fn relative_from<B: AsRef<Path>>(&self, path: B) -> Option<PathBuf>;
}

impl PathExt for Path {
    fn relative_from<B: AsRef<Path>>(&self, path: B) -> Option<PathBuf> {
        let path = path.as_ref();

        if self.is_absolute() != path.is_absolute() {
            if self.is_absolute() {
                return Some(PathBuf::from(self));
            } else {
                return None;
            }
        }

        let mut itlhs = self.components();
        let mut itrhs = path.components();

        let mut components: Vec<Component> = Vec::new();

        loop {
            match (itlhs.next(), itrhs.next()) {
                (None, None) => break,
                (Some(lhs), None) => {
                    components.push(lhs);
                    components.extend(itlhs.by_ref());
                    break;
                }
                (None, _) => components.push(Component::ParentDir),
                (Some(lhs), Some(rhs)) if components.is_empty() && lhs == rhs => (),
                (Some(lhs), Some(Component::CurDir)) => components.push(lhs),
                (Some(_), Some(Component::ParentDir)) => return None,
                (Some(lhs), Some(_)) => {
                    components.push(Component::ParentDir);

                    for _ in itrhs {
                        components.push(Component::ParentDir);
                    }

                    components.push(lhs);
                    components.extend(itlhs.by_ref());
                    break;
                }
            }
        }

        Some(
            components
                .into_iter()
                .map(|component| component.as_os_str())
                .collect(),
        )
    }
}

impl PathExt for PathBuf {
    fn relative_from<B: AsRef<Path>>(&self, path: B) -> Option<PathBuf> {
        self.as_path().relative_from(path)
    }
}
