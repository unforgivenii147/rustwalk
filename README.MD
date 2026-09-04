# rustwalk
i originally named it rustwalk, but it was taken 
so i named it rustwalk

# install
maturin build

# usage
it accept Path or string input 
and returns list of Path
```python
from rustwalk import walk_parallel
for path in walk_parallel("/"):
   print(path.name)

```

BUILD:
LIBRARY_PATH=~/.local/lib:$LIBRARY_PATH LD_LIBRARY_PATH=~/.local/lib:$LD_LIBRARY_PATH RUSTFLAGS="-L ~/.local/lib" python -m maturin build --verbose
