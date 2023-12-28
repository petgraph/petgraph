//! [`Harness`] for discovering test inputs and asserting against snapshot files
//!
//! Taken from [snapbox](https://docs.rs/snapshot) and adapted to make matrix tests possible.

use ignore::{
    overrides::{Override, OverrideBuilder},
    WalkBuilder,
};
use libtest_mimic::Trial;
use snapbox::{
    report::{write_diff, Palette},
    Action, Data, DataFormat, NormalizeNewlines,
};

pub struct Harness<S, T> {
    root: std::path::PathBuf,
    overrides: Option<Override>,
    each: Option<&'static [&'static str]>,
    setup: S,
    test: T,
    action: Action,
}

impl<S, T, I, E> Harness<S, T>
where
    I: std::fmt::Display,
    E: std::fmt::Display,
    S: Fn(std::path::PathBuf) -> Case + Send + Sync + 'static,
    T: Fn(&std::path::Path, &'static str) -> Result<I, E> + Send + Sync + 'static + Clone,
{
    pub fn new(root: impl Into<std::path::PathBuf>, setup: S, test: T) -> Self {
        Self {
            root: root.into(),
            overrides: None,
            setup,
            // in theory we would want to do this via a type-state instead,
            // but I can't be bothered to put in the extra effort, unless we upstream this.
            each: None,
            test,
            action: Action::Verify,
        }
    }

    /// Path patterns for selecting input files
    ///
    /// This used gitignore syntax
    pub fn select<'p>(mut self, patterns: impl IntoIterator<Item = &'p str>) -> Self {
        let mut overrides = OverrideBuilder::new(&self.root);
        for line in patterns {
            overrides.add(line).unwrap();
        }
        self.overrides = Some(overrides.build().unwrap());
        self
    }

    pub fn each(mut self, names: &'static [&'static str]) -> Self {
        self.each = Some(names);
        self
    }

    /// Read the failure action from an environment variable
    pub fn action_env(mut self, var_name: &str) -> Self {
        let action = Action::with_env_var(var_name);
        self.action = action.unwrap_or(self.action);
        self
    }

    /// Override the failure action
    pub fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    fn trials(&self, name: &'static str) -> impl IntoIterator<Item = Trial> + '_ {
        let mut walk = WalkBuilder::new(&self.root);
        walk.standard_filters(false);
        let tests = walk.build().filter_map(|entry| {
            let entry = entry.unwrap();
            let is_dir = entry.file_type().map(|f| f.is_dir()).unwrap_or(false);
            let path = entry.into_path();
            if let Some(overrides) = &self.overrides {
                overrides
                    .matched(&path, is_dir)
                    .is_whitelist()
                    .then_some(path)
            } else {
                Some(path)
            }
        });

        tests.into_iter().map(move |path| {
            let case = (self.setup)(path);

            let test = self.test.clone();
            let trial_name = if name.is_empty() {
                case.name.clone()
            } else {
                format!("{name}::{}", case.name)
            };
            let action = self.action;

            Trial::test(trial_name, move || {
                let actual = test(&case.fixture, name)?;
                let actual = actual.to_string();
                let actual = Data::text(actual).normalize(NormalizeNewlines);

                let verify = Verifier::new().palette(Palette::color()).action(action);
                verify.verify(&case.expected, actual)?;
                Ok(())
            })
            .with_ignored_flag(action == Action::Ignore)
        })
    }

    /// Run tests
    pub fn test(self) -> ! {
        let each = self.each.unwrap_or(&[""]);

        let tests = each.iter().flat_map(|name| self.trials(name)).collect();

        let args = libtest_mimic::Arguments::from_args();
        libtest_mimic::run(&args, tests).exit()
    }
}

struct Verifier {
    palette: Palette,
    action: Action,
}

impl Verifier {
    fn new() -> Self {
        Default::default()
    }

    fn palette(mut self, palette: Palette) -> Self {
        self.palette = palette;
        self
    }

    fn action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    fn verify(&self, expected_path: &std::path::Path, actual: Data) -> snapbox::Result<()> {
        match self.action {
            Action::Skip => Ok(()),
            Action::Ignore => {
                let _ = self.try_verify(expected_path, actual);
                Ok(())
            }
            Action::Verify => self.try_verify(expected_path, actual),
            Action::Overwrite => self.try_overwrite(expected_path, actual),
        }
    }

    fn try_overwrite(&self, expected_path: &std::path::Path, actual: Data) -> snapbox::Result<()> {
        actual.write_to(expected_path)?;
        Ok(())
    }

    fn try_verify(&self, expected_path: &std::path::Path, actual: Data) -> snapbox::Result<()> {
        let expected =
            Data::read_from(expected_path, Some(DataFormat::Text))?.normalize(NormalizeNewlines);

        if expected != actual {
            let mut buf = String::new();
            write_diff(
                &mut buf,
                &expected,
                &actual,
                Some(&expected_path.display()),
                None,
                self.palette,
            )
            .map_err(|e| e.to_string())?;
            Err(buf.into())
        } else {
            Ok(())
        }
    }
}

impl Default for Verifier {
    fn default() -> Self {
        Self {
            palette: Palette::color(),
            action: Action::Verify,
        }
    }
}

pub struct Case {
    pub name: String,
    pub fixture: std::path::PathBuf,
    pub expected: std::path::PathBuf,
}
