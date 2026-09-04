use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::PathBuf;
use walkdir::WalkDir;

// Directories to skip during traversal
const SKIP_DIRS: &[&str] = &[".git", "__pycache__", ".mypy_cache", "lazy"];

fn to_py_path<'py>(py: Python<'py>, path: PathBuf) -> PyResult<&'py PyAny> {
    let pathlib = py.import("pathlib")?;
    pathlib.call_method1("Path", (path,))
}

fn should_skip(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| SKIP_DIRS.contains(&s))
        .unwrap_or(false)
}

#[derive(Clone)]
struct PathInput(PathBuf);

impl<'py> FromPyObject<'py> for PathInput {
    fn extract(obj: &'py PyAny) -> PyResult<Self> {
        if let Ok(s) = obj.extract::<String>() {
            return Ok(PathInput(PathBuf::from(s)));
        }

        if let Ok(path_str) = obj.call_method0("__str__") {
            if let Ok(s) = path_str.extract::<String>() {
                return Ok(PathInput(PathBuf::from(s)));
            }
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a string or pathlib.Path object",
        ))
    }
}

impl From<PathInput> for PathBuf {
    fn from(input: PathInput) -> Self {
        input.0
    }
}

// Helper function to create an iterator with filters applied
fn create_filtered_iterator(path_buf: &PathBuf, follow_links: bool) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> {
    WalkDir::new(path_buf)
        .follow_links(follow_links)
        .into_iter()
        .filter_entry(move |e| {
            // Skip unwanted directories
            if should_skip(e) {
                return false;
            }
            // Skip symlinks if not following links
            if !follow_links && e.file_type().is_symlink() {
                return false;
            }
            true
        })
}

#[pyfunction]
fn walk(py: Python<'_>, path: PathInput, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let walker = create_filtered_iterator(&path_buf, follow);
    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                results.push(to_py_path(py, e.path().to_path_buf())?);
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pyfunction]
fn walk_files(
    py: Python<'_>,
    path: PathInput,
    follow_links: Option<bool>,
) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let walker = create_filtered_iterator(&path_buf, follow);
    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                if e.file_type().is_file() {
                    results.push(to_py_path(py, e.path().to_path_buf())?);
                }
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pyfunction]
fn walk_dirs(py: Python<'_>, path: PathInput, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let walker = create_filtered_iterator(&path_buf, follow);
    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                if e.file_type().is_dir() {
                    results.push(to_py_path(py, e.path().to_path_buf())?);
                }
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pyclass]
#[derive(Clone)]
struct Entry {
    #[pyo3(get)]
    path: Py<PyAny>,
    #[pyo3(get)]
    is_file: bool,
    #[pyo3(get)]
    is_dir: bool,
    #[pyo3(get)]
    is_symlink: bool,
    #[pyo3(get)]
    depth: usize,
}

#[pyfunction]
fn walk_with_metadata(
    py: Python<'_>,
    path: PathInput,
    follow_links: Option<bool>,
    max_depth: Option<usize>,
    min_depth: Option<usize>,
) -> PyResult<Vec<Entry>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    
    let mut walker = WalkDir::new(path_buf).follow_links(follow);
    if let Some(max) = max_depth {
        walker = walker.max_depth(max);
    }
    if let Some(min) = min_depth {
        walker = walker.min_depth(min);
    }
    
    let walker = walker.into_iter().filter_entry(|e| {
        // Skip unwanted directories
        if should_skip(e) {
            return false;
        }
        // Skip symlinks if not following links
        if !follow && e.file_type().is_symlink() {
            return false;
        }
        true
    });

    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                let file_type = e.file_type();
                let py_path = to_py_path(py, e.path().to_path_buf())?.into();
                results.push(Entry {
                    path: py_path,
                    is_file: file_type.is_file(),
                    is_dir: file_type.is_dir(),
                    is_symlink: file_type.is_symlink(),
                    depth: e.depth(),
                });
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

// Note: walkdir doesn't have built-in parallelism, so walk_parallel
// behaves the same as walk (single-threaded)
#[pyfunction]
fn walk_parallel(
    py: Python<'_>,
    path: PathInput,
    follow_links: Option<bool>,
    num_threads: Option<usize>,
) -> PyResult<Vec<&PyAny>> {
    // num_threads is ignored since walkdir doesn't support parallelism
    // Keep the parameter for API compatibility
    let _ = num_threads;
    
    walk(py, path, follow_links)
}

#[pymodule]
fn fastwalk(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(walk, m)?)?;
    m.add_function(wrap_pyfunction!(walk_files, m)?)?;
    m.add_function(wrap_pyfunction!(walk_dirs, m)?)?;
    m.add_function(wrap_pyfunction!(walk_with_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(walk_parallel, m)?)?;
    m.add_class::<Entry>()?;
    Ok(())
}