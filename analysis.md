# unknown-project AST Analysis

**Version:** 0.1.0
**Rust Edition:** 2021
**Extracted:** 2025-07-09 20:25:45 UTC

## Project Metrics

- **Total Files:** 10
- **Total Lines:** 939
- **Functions:** 13
- **Structs:** 14
- **Enums:** 5
- **Traits:** 0
- **Average Complexity:** 1.16

## Files

### crates/rustex-cli/src/main.rs

#### Struct `Cli`

#### Enum `Commands`

#### Enum `CliOutputFormat`

#### Function `main`

```rust
# [tokio :: main] async fn main () -> Result < () > { let cli = Cli :: parse () ; let log_level = if cli . verbose { "debug" } else { "info" } ; tracing_subscriber :: fmt () . with_env_filter (format ! ("rustex={}" , log_level)) . init () ; match cli . command { Commands :: Extract { format , output , include_docs , include_private , parse_deps , max_file_size , include , exclude , plugins , pretty , } => { extract_command (cli . path , format . into () , output , include_docs , include_private , parse_deps , max_file_size , include , exclude , plugins , pretty ,) . await ? ; } Commands :: Deps { visualize , output } => { deps_command (cli . path , visualize , output) . await ? ; } Commands :: Metrics { complexity , loc , output } => { metrics_command (cli . path , complexity , loc , output) . await ? ; } Commands :: Init { force } => { init_command (cli . path , force) . await ? ; } } Ok (()) } . sig
```

#### Function `extract_command`

```rust
async fn extract_command (project_path : PathBuf , format : OutputFormat , output : Option < PathBuf > , include_docs : bool , include_private : bool , parse_deps : bool , max_file_size : usize , include_patterns : Vec < String > , exclude_patterns : Vec < String > , plugins : Vec < String > , pretty : bool ,) -> Result < () > { info ! ("Starting AST extraction for project at {:?}" , project_path) ; let config = create_config (format . clone () , include_docs , include_private , parse_deps , max_file_size , include_patterns , exclude_patterns , plugins ,) ; let extractor = AstExtractor :: new (config , project_path) ; let pb = indicatif :: ProgressBar :: new_spinner () ; pb . set_message ("Extracting AST...") ; pb . enable_steady_tick (std :: time :: Duration :: from_millis (100)) ; match extractor . extract_project () { Ok (ast_data) => { pb . finish_with_message ("✓ AST extraction completed") ; let output_content = match format { OutputFormat :: Json => { if pretty { serde_json :: to_string_pretty (& ast_data) ? } else { serde_json :: to_string (& ast_data) ? } } OutputFormat :: Markdown => { generate_markdown_output (& ast_data) ? } _ => { error ! ("Output format not yet implemented") ; return Ok (()) ; } } ; match output { Some (path) => { std :: fs :: write (& path , output_content) ? ; println ! ("✓ Output written to {}" , path . display ()) ; } None => { println ! ("{}" , output_content) ; } } print_extraction_summary (& ast_data) ; } Err (e) => { pb . finish_with_message ("✗ AST extraction failed") ; error ! ("Extraction failed: {}" , e) ; return Err (e) ; } } Ok (()) } . sig
```

#### Function `deps_command`

```rust
async fn deps_command (_project_path : PathBuf , _visualize : bool , _output : Option < PathBuf > ,) -> Result < () > { println ! ("🚧 Dependency analysis not yet implemented") ; Ok (()) } . sig
```

#### Function `metrics_command`

```rust
async fn metrics_command (_project_path : PathBuf , _complexity : bool , _loc : bool , _output : Option < PathBuf > ,) -> Result < () > { println ! ("🚧 Metrics analysis not yet implemented") ; Ok (()) } . sig
```

#### Function `init_command`

```rust
async fn init_command (project_path : PathBuf , force : bool) -> Result < () > { let config_path = project_path . join ("rustex.toml") ; if config_path . exists () && ! force { error ! ("Configuration file already exists. Use --force to overwrite.") ; return Ok (()) ; } let default_config = r#"[extraction]
include_docs = true
include_private = false
parse_dependencies = false
max_file_size = "10MB"
output_format = "json"

[filters]
include = ["src/**/*.rs", "lib/**/*.rs"]
exclude = ["target/**", "tests/**"]

[plugins]
enabled = []

# Plugin configurations can be added here
"# ; std :: fs :: write (& config_path , default_config) ? ; println ! ("✓ Created configuration file at {}" , config_path . display ()) ; Ok (()) } . sig
```

#### Function `create_config`

```rust
fn create_config (format : OutputFormat , include_docs : bool , include_private : bool , parse_deps : bool , max_file_size : usize , include_patterns : Vec < String > , exclude_patterns : Vec < String > , plugins : Vec < String > ,) -> ExtractorConfig { let mut config = ExtractorConfig :: default () ; config . output_format = format ; config . include_docs = include_docs ; config . include_private = include_private ; config . parse_dependencies = parse_deps ; config . max_file_size = max_file_size ; config . plugins = plugins ; if ! include_patterns . is_empty () { config . filters . include = include_patterns ; } if ! exclude_patterns . is_empty () { config . filters . exclude = exclude_patterns ; } config } . sig
```

#### Function `generate_markdown_output`

**Documentation:**
>  Generate markdown output from AST data.

```rust
# [doc = " Generate markdown output from AST data."] fn generate_markdown_output (ast_data : & rustex_core :: ProjectAst) -> Result < String > { let mut output = String :: new () ; output . push_str (& format ! ("# {} AST Analysis\n\n" , ast_data . project . name)) ; output . push_str (& format ! ("**Version:** {}\n" , ast_data . project . version)) ; output . push_str (& format ! ("**Rust Edition:** {}\n" , ast_data . project . rust_edition)) ; output . push_str (& format ! ("**Extracted:** {}\n\n" , ast_data . extracted_at . format ("%Y-%m-%d %H:%M:%S UTC"))) ; output . push_str ("## Project Metrics\n\n") ; output . push_str (& format ! ("- **Total Files:** {}\n" , ast_data . metrics . total_files)) ; output . push_str (& format ! ("- **Total Lines:** {}\n" , ast_data . metrics . total_lines)) ; output . push_str (& format ! ("- **Functions:** {}\n" , ast_data . metrics . total_functions)) ; output . push_str (& format ! ("- **Structs:** {}\n" , ast_data . metrics . total_structs)) ; output . push_str (& format ! ("- **Enums:** {}\n" , ast_data . metrics . total_enums)) ; output . push_str (& format ! ("- **Traits:** {}\n" , ast_data . metrics . total_traits)) ; output . push_str (& format ! ("- **Average Complexity:** {:.2}\n\n" , ast_data . metrics . complexity_average)) ; if ! ast_data . files . is_empty () { output . push_str ("## Files\n\n") ; for file in & ast_data . files { output . push_str (& format ! ("### {}\n\n" , file . relative_path . display ())) ; if ! file . elements . is_empty () { for element in & file . elements { output . push_str (& format ! ("#### {} `{}`\n\n" , format ! ("{:?}" , element . element_type) , element . name)) ; if ! element . doc_comments . is_empty () { output . push_str ("**Documentation:**\n") ; for doc in & element . doc_comments { output . push_str (& format ! ("> {}\n" , doc)) ; } output . push_str ("\n") ; } if let Some (ref signature) = element . signature { output . push_str (& format ! ("```rust\n{}\n```\n\n" , signature)) ; } } } else { output . push_str ("*No extractable elements found*\n\n") ; } } } Ok (output) } . sig
```

#### Function `print_extraction_summary`

**Documentation:**
>  Print extraction summary to terminal.

```rust
# [doc = " Print extraction summary to terminal."] fn print_extraction_summary (ast_data : & rustex_core :: ProjectAst) { use colored :: * ; println ! ("\n{}" , "📊 Extraction Summary" . bold () . green ()) ; println ! ("{}" , "─" . repeat (50)) ; println ! ("📁 Project: {}" , ast_data . project . name . cyan ()) ; println ! ("📄 Files processed: {}" , ast_data . metrics . total_files . to_string () . yellow ()) ; println ! ("📏 Total lines: {}" , ast_data . metrics . total_lines . to_string () . yellow ()) ; println ! ("\n{}" , "🔍 Code Elements:" . bold ()) ; println ! ("  🔧 Functions: {}" , ast_data . metrics . total_functions . to_string () . blue ()) ; println ! ("  🏗️  Structs: {}" , ast_data . metrics . total_structs . to_string () . blue ()) ; println ! ("  🎯 Enums: {}" , ast_data . metrics . total_enums . to_string () . blue ()) ; println ! ("  🎭 Traits: {}" , ast_data . metrics . total_traits . to_string () . blue ()) ; if ast_data . metrics . complexity_average > 0.0 { let complexity_color = if ast_data . metrics . complexity_average > 10.0 { "red" } else if ast_data . metrics . complexity_average > 5.0 { "yellow" } else { "green" } ; let formatted_complexity = format ! ("{:.2}" , ast_data . metrics . complexity_average) ; println ! ("📈 Avg. Complexity: {}" , match complexity_color { "red" => formatted_complexity . red () , "yellow" => formatted_complexity . yellow () , _ => formatted_complexity . green () , }) ; } println ! () ; } . sig
```

### crates/rustex-core/src/extractor.rs

#### Struct `AstExtractor`

**Documentation:**
>  Main AST extractor for Rust projects.

#### Function `glob_match`

**Documentation:**
>  Simple glob pattern matching (simplified implementation).

```rust
# [doc = " Simple glob pattern matching (simplified implementation)."] fn glob_match (pattern : & str , text : & str) -> bool { if pattern . contains ("**") { let prefix = pattern . split ("**") . next () . unwrap_or ("") ; text . contains (prefix) } else if pattern . ends_with ("*") { let prefix = & pattern [.. pattern . len () - 1] ; text . starts_with (prefix) } else { text . contains (pattern) } } . sig
```

#### Function `extract_imports`

**Documentation:**
>  Extract imports from a Rust file.

```rust
# [doc = " Extract imports from a Rust file."] fn extract_imports (file : & syn :: File) -> Vec < ImportInfo > { let mut imports = Vec :: new () ; for item in & file . items { if let syn :: Item :: Use (use_item) = item { if let Some (import_info) = parse_use_tree (& use_item . tree) { imports . push (import_info) ; } } } imports } . sig
```

#### Function `parse_use_tree`

**Documentation:**
>  Parse a use tree into import information.

```rust
# [doc = " Parse a use tree into import information."] fn parse_use_tree (tree : & syn :: UseTree) -> Option < ImportInfo > { match tree { syn :: UseTree :: Path (path) => { Some (ImportInfo { module_path : path . ident . to_string () , imported_items : vec ! [] , is_glob : false , alias : None , }) } syn :: UseTree :: Name (name) => { Some (ImportInfo { module_path : "" . to_string () , imported_items : vec ! [name . ident . to_string ()] , is_glob : false , alias : None , }) } syn :: UseTree :: Glob (_) => { Some (ImportInfo { module_path : "" . to_string () , imported_items : vec ! [] , is_glob : true , alias : None , }) } syn :: UseTree :: Group (group) => { let items : Vec < String > = group . items . iter () . filter_map (| item | { if let syn :: UseTree :: Name (name) = item { Some (name . ident . to_string ()) } else { None } }) . collect () ; Some (ImportInfo { module_path : "" . to_string () , imported_items : items , is_glob : false , alias : None , }) } syn :: UseTree :: Rename (rename) => { Some (ImportInfo { module_path : "" . to_string () , imported_items : vec ! [rename . ident . to_string ()] , is_glob : false , alias : Some (rename . rename . to_string ()) , }) } } } . sig
```

#### Function `calculate_file_metrics`

**Documentation:**
>  Calculate metrics for a file.

```rust
# [doc = " Calculate metrics for a file."] fn calculate_file_metrics (content : & str , elements : & [CodeElement]) -> FileMetrics { let lines : Vec < & str > = content . lines () . collect () ; let lines_of_code = lines . iter () . filter (| line | ! line . trim () . is_empty () && ! line . trim () . starts_with ("//")) . count () ; let lines_of_comments = lines . iter () . filter (| line | line . trim () . starts_with ("//")) . count () ; FileMetrics { lines_of_code , lines_of_comments , complexity_total : elements . iter () . map (| e | e . complexity . unwrap_or (0)) . sum () , function_count : elements . iter () . filter (| e | matches ! (e . element_type , ElementType :: Function)) . count () , struct_count : elements . iter () . filter (| e | matches ! (e . element_type , ElementType :: Struct)) . count () , enum_count : elements . iter () . filter (| e | matches ! (e . element_type , ElementType :: Enum)) . count () , trait_count : elements . iter () . filter (| e | matches ! (e . element_type , ElementType :: Trait)) . count () , } } . sig
```

#### Function `extract_toml_field`

**Documentation:**
>  Extract a field value from TOML content (simplified parser).

```rust
# [doc = " Extract a field value from TOML content (simplified parser)."] fn extract_toml_field (content : & str , field : & str) -> Option < String > { for line in content . lines () { let line = line . trim () ; if line . starts_with (& format ! ("{} =" , field)) { if let Some (value_part) = line . split ('=') . nth (1) { let value = value_part . trim () . trim_matches ('"') . trim_matches ('\'') ; return Some (value . to_string ()) ; } } } None } . sig
```

### crates/rustex-core/src/visitors.rs

#### Struct `CodeElementVisitor`

**Documentation:**
>  Visitor for extracting code elements from Rust AST.

### crates/rustex-core/src/config.rs

#### Struct `ExtractorConfig`

**Documentation:**
>  Configuration for AST extraction.

#### Enum `OutputFormat`

**Documentation:**
>  Available output formats for AST data.

#### Struct `FilterConfig`

**Documentation:**
>  File filtering configuration.

### crates/rustex-core/src/lib.rs

*No extractable elements found*

### crates/rustex-core/src/ast_data.rs

#### Struct `ProjectAst`

**Documentation:**
>  Complete AST representation of a Rust project.

#### Struct `ProjectInfo`

**Documentation:**
>  Project metadata information.

#### Struct `FileAst`

**Documentation:**
>  AST representation of a single Rust file.

#### Struct `CodeElement`

**Documentation:**
>  A single code element (function, struct, etc.).

#### Enum `ElementType`

**Documentation:**
>  Types of code elements that can be extracted.

#### Enum `Visibility`

**Documentation:**
>  Visibility levels for code elements.

#### Struct `CodeLocation`

**Documentation:**
>  Location information for code elements.

#### Struct `ImportInfo`

**Documentation:**
>  Import/use statement information.

#### Struct `DependencyInfo`

**Documentation:**
>  Dependency information for the project.

#### Struct `ProjectMetrics`

**Documentation:**
>  Project-wide metrics.

#### Struct `FileMetrics`

**Documentation:**
>  File-level metrics.

### crates/rustex-plugins/src/lib.rs

*No extractable elements found*

### crates/rustex-plugins/src/plugin.rs

*No extractable elements found*

### crates/rustex-formats/src/lib.rs

*No extractable elements found*

### crates/rustex-formats/src/formatters.rs

*No extractable elements found*

