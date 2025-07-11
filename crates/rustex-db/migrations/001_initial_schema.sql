-- RustEx Database Schema v1.0
-- Optimized for graph traversal and AST storage

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For text search
CREATE EXTENSION IF NOT EXISTS "btree_gin"; -- For compound indexes

-- Create custom types
CREATE TYPE element_type AS ENUM (
    'Function', 'Struct', 'Enum', 'Trait', 'Module', 'Impl', 'Use', 
    'Macro', 'Const', 'Static', 'TypeAlias', 'ExternCrate', 'Field', 
    'Variant', 'TraitItem', 'ImplItem', 'Mod', 'ForeignItem'
);

CREATE TYPE visibility_type AS ENUM ('Public', 'Private', 'Crate', 'Restricted');
CREATE TYPE reference_type AS ENUM ('FunctionCall', 'TypeUsage', 'VariableRef', 'ModuleRef', 'TraitRef');
CREATE TYPE dependency_type AS ENUM ('Uses', 'Calls', 'Implements', 'Extends', 'Contains', 'Imports');
CREATE TYPE call_type AS ENUM ('Direct', 'Indirect', 'Virtual', 'Trait', 'Macro');
CREATE TYPE relationship_type AS ENUM ('Implements', 'Extends', 'Uses', 'Contains', 'Composes', 'Aggregates');

-- Projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    rust_edition VARCHAR(10) NOT NULL DEFAULT '2021',
    description TEXT,
    authors TEXT[] DEFAULT '{}',
    license VARCHAR(100),
    repository_url TEXT,
    homepage TEXT,
    keywords TEXT[] DEFAULT '{}',
    categories TEXT[] DEFAULT '{}',
    readme_path TEXT,
    build_script TEXT,
    workspace_root TEXT,
    target_directory TEXT,
    features JSONB DEFAULT '{}',
    dependencies JSONB DEFAULT '{}',
    dev_dependencies JSONB DEFAULT '{}',
    build_dependencies JSONB DEFAULT '{}',
    
    -- Metrics
    total_files INTEGER DEFAULT 0,
    total_lines BIGINT DEFAULT 0,
    total_functions INTEGER DEFAULT 0,
    total_structs INTEGER DEFAULT 0,
    total_enums INTEGER DEFAULT 0,
    total_traits INTEGER DEFAULT 0,
    total_modules INTEGER DEFAULT 0,
    total_impls INTEGER DEFAULT 0,
    complexity_average DECIMAL(10,2) DEFAULT 0.0,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    analyzed_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Constraints
    CONSTRAINT projects_name_version_unique UNIQUE (name, version)
);

-- Files table
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    extension VARCHAR(10) NOT NULL DEFAULT 'rs',
    size_bytes BIGINT DEFAULT 0,
    
    -- Code metrics
    lines_of_code INTEGER DEFAULT 0,
    function_count INTEGER DEFAULT 0,
    struct_count INTEGER DEFAULT 0,
    enum_count INTEGER DEFAULT 0,
    trait_count INTEGER DEFAULT 0,
    module_count INTEGER DEFAULT 0,
    impl_count INTEGER DEFAULT 0,
    use_count INTEGER DEFAULT 0,
    macro_count INTEGER DEFAULT 0,
    const_count INTEGER DEFAULT 0,
    static_count INTEGER DEFAULT 0,
    type_alias_count INTEGER DEFAULT 0,
    
    -- Complexity metrics
    complexity_total INTEGER DEFAULT 0,
    complexity_average DECIMAL(10,2) DEFAULT 0.0,
    documentation_coverage DECIMAL(5,2) DEFAULT 0.0,
    test_coverage DECIMAL(5,2),
    
    -- File metadata
    last_modified TIMESTAMPTZ DEFAULT NOW(),
    analyzed_at TIMESTAMPTZ DEFAULT NOW(),
    content_hash VARCHAR(64),
    syntax_errors TEXT[] DEFAULT '{}',
    warnings TEXT[] DEFAULT '{}',
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Constraints
    CONSTRAINT files_project_path_unique UNIQUE (project_id, relative_path)
);

-- AST Elements table
CREATE TABLE ast_elements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    element_id VARCHAR(255) NOT NULL, -- Original element ID from AST
    element_type element_type NOT NULL,
    name VARCHAR(255) NOT NULL,
    qualified_name TEXT NOT NULL,
    signature TEXT,
    visibility visibility_type NOT NULL DEFAULT 'Private',
    
    -- Location information
    line_start INTEGER NOT NULL,
    line_end INTEGER NOT NULL,
    char_start INTEGER NOT NULL,
    char_end INTEGER NOT NULL,
    
    -- Complexity metrics
    complexity INTEGER,
    cyclomatic_complexity INTEGER,
    cognitive_complexity INTEGER,
    nesting_depth INTEGER,
    parameter_count INTEGER,
    return_count INTEGER,
    lines_of_code INTEGER,
    halstead_metrics JSONB,
    
    -- Documentation and comments
    doc_comments TEXT[] DEFAULT '{}',
    inline_comments TEXT[] DEFAULT '{}',
    attributes TEXT[] DEFAULT '{}',
    dependencies TEXT[] DEFAULT '{}',
    generic_params TEXT[] DEFAULT '{}',
    
    -- Hierarchy information
    module_path TEXT NOT NULL,
    parent_element_id UUID REFERENCES ast_elements(id),
    nesting_level INTEGER DEFAULT 0,
    
    -- Boolean flags
    is_public BOOLEAN DEFAULT FALSE,
    is_test BOOLEAN DEFAULT FALSE,
    is_async BOOLEAN DEFAULT FALSE,
    is_unsafe BOOLEAN DEFAULT FALSE,
    is_deprecated BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Constraints
    CONSTRAINT ast_elements_project_element_id_unique UNIQUE (project_id, element_id)
);

-- Cross-references table for tracking references between elements
CREATE TABLE cross_references (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    from_element_id UUID NOT NULL REFERENCES ast_elements(id) ON DELETE CASCADE,
    to_element_id UUID REFERENCES ast_elements(id) ON DELETE CASCADE,
    reference_type reference_type NOT NULL,
    reference_text VARCHAR(255) NOT NULL,
    line_number INTEGER NOT NULL,
    char_position INTEGER NOT NULL,
    context_scope TEXT,
    is_definition BOOLEAN DEFAULT FALSE,
    is_resolved BOOLEAN DEFAULT FALSE,
    confidence_score DECIMAL(3,2) DEFAULT 1.0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Dependencies table for element relationships
CREATE TABLE dependencies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    from_element_id UUID NOT NULL REFERENCES ast_elements(id) ON DELETE CASCADE,
    to_element_id UUID NOT NULL REFERENCES ast_elements(id) ON DELETE CASCADE,
    dependency_type dependency_type NOT NULL,
    strength DECIMAL(3,2) DEFAULT 1.0,
    is_direct BOOLEAN DEFAULT TRUE,
    is_cyclic BOOLEAN DEFAULT FALSE,
    path_length INTEGER DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    
    -- Prevent duplicate dependencies
    CONSTRAINT dependencies_unique UNIQUE (from_element_id, to_element_id, dependency_type)
);

-- Call chains table for function call relationships
CREATE TABLE call_chains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    caller_id UUID NOT NULL REFERENCES ast_elements(id) ON DELETE CASCADE,
    callee_id UUID NOT NULL REFERENCES ast_elements(id) ON DELETE CASCADE,
    call_type call_type NOT NULL DEFAULT 'Direct',
    call_count INTEGER DEFAULT 1,
    call_sites INTEGER[] DEFAULT '{}', -- Line numbers where calls occur
    is_recursive BOOLEAN DEFAULT FALSE,
    recursion_depth INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    
    -- Prevent duplicate call chains
    CONSTRAINT call_chains_unique UNIQUE (caller_id, callee_id, call_type)
);

-- Type relationships table for struct/enum/trait relationships
CREATE TABLE type_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    from_type_id UUID NOT NULL REFERENCES ast_elements(id) ON DELETE CASCADE,
    to_type_id UUID NOT NULL REFERENCES ast_elements(id) ON DELETE CASCADE,
    relationship_type relationship_type NOT NULL,
    relationship_strength DECIMAL(3,2) DEFAULT 1.0,
    is_generic BOOLEAN DEFAULT FALSE,
    generic_constraints TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    
    -- Prevent duplicate relationships
    CONSTRAINT type_relationships_unique UNIQUE (from_type_id, to_type_id, relationship_type)
);

-- Indexes for optimal query performance

-- Project indexes
CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_created_at ON projects(created_at);
CREATE INDEX idx_projects_metadata ON projects USING GIN(metadata);

-- File indexes
CREATE INDEX idx_files_project_id ON files(project_id);
CREATE INDEX idx_files_path_trgm ON files USING GIN(relative_path gin_trgm_ops);
CREATE INDEX idx_files_extension ON files(extension);
CREATE INDEX idx_files_analyzed_at ON files(analyzed_at);
CREATE INDEX idx_files_metadata ON files USING GIN(metadata);

-- AST Elements indexes (optimized for graph traversal)
CREATE INDEX idx_ast_elements_project_id ON ast_elements(project_id);
CREATE INDEX idx_ast_elements_file_id ON ast_elements(file_id);
CREATE INDEX idx_ast_elements_type ON ast_elements(element_type);
CREATE INDEX idx_ast_elements_name_trgm ON ast_elements USING GIN(name gin_trgm_ops);
CREATE INDEX idx_ast_elements_qualified_name_trgm ON ast_elements USING GIN(qualified_name gin_trgm_ops);
CREATE INDEX idx_ast_elements_visibility ON ast_elements(visibility);
CREATE INDEX idx_ast_elements_parent ON ast_elements(parent_element_id);
CREATE INDEX idx_ast_elements_module_path ON ast_elements(module_path);
CREATE INDEX idx_ast_elements_complexity ON ast_elements(complexity);
CREATE INDEX idx_ast_elements_public ON ast_elements(is_public) WHERE is_public = TRUE;
CREATE INDEX idx_ast_elements_test ON ast_elements(is_test) WHERE is_test = TRUE;
CREATE INDEX idx_ast_elements_metadata ON ast_elements USING GIN(metadata);

-- Compound indexes for common queries
CREATE INDEX idx_ast_elements_project_type ON ast_elements(project_id, element_type);
CREATE INDEX idx_ast_elements_file_type ON ast_elements(file_id, element_type);
CREATE INDEX idx_ast_elements_project_name ON ast_elements(project_id, name);

-- Cross-references indexes (critical for graph traversal)
CREATE INDEX idx_cross_references_project_id ON cross_references(project_id);
CREATE INDEX idx_cross_references_from_element ON cross_references(from_element_id);
CREATE INDEX idx_cross_references_to_element ON cross_references(to_element_id);
CREATE INDEX idx_cross_references_type ON cross_references(reference_type);
CREATE INDEX idx_cross_references_resolved ON cross_references(is_resolved);

-- Compound indexes for cross-reference queries
CREATE INDEX idx_cross_references_from_type ON cross_references(from_element_id, reference_type);
CREATE INDEX idx_cross_references_to_type ON cross_references(to_element_id, reference_type);
CREATE INDEX idx_cross_references_project_type ON cross_references(project_id, reference_type);

-- Dependencies indexes (for dependency graph traversal)
CREATE INDEX idx_dependencies_project_id ON dependencies(project_id);
CREATE INDEX idx_dependencies_from_element ON dependencies(from_element_id);
CREATE INDEX idx_dependencies_to_element ON dependencies(to_element_id);
CREATE INDEX idx_dependencies_type ON dependencies(dependency_type);
CREATE INDEX idx_dependencies_direct ON dependencies(is_direct);
CREATE INDEX idx_dependencies_cyclic ON dependencies(is_cyclic);

-- Compound indexes for dependency traversal
CREATE INDEX idx_dependencies_from_type ON dependencies(from_element_id, dependency_type);
CREATE INDEX idx_dependencies_to_type ON dependencies(to_element_id, dependency_type);
CREATE INDEX idx_dependencies_project_type ON dependencies(project_id, dependency_type);

-- Call chains indexes (for call graph analysis)
CREATE INDEX idx_call_chains_project_id ON call_chains(project_id);
CREATE INDEX idx_call_chains_caller ON call_chains(caller_id);
CREATE INDEX idx_call_chains_callee ON call_chains(callee_id);
CREATE INDEX idx_call_chains_type ON call_chains(call_type);
CREATE INDEX idx_call_chains_recursive ON call_chains(is_recursive) WHERE is_recursive = TRUE;

-- Compound indexes for call chain analysis
CREATE INDEX idx_call_chains_caller_type ON call_chains(caller_id, call_type);
CREATE INDEX idx_call_chains_callee_type ON call_chains(callee_id, call_type);

-- Type relationships indexes
CREATE INDEX idx_type_relationships_project_id ON type_relationships(project_id);
CREATE INDEX idx_type_relationships_from_type ON type_relationships(from_type_id);
CREATE INDEX idx_type_relationships_to_type ON type_relationships(to_type_id);
CREATE INDEX idx_type_relationships_type ON type_relationships(relationship_type);
CREATE INDEX idx_type_relationships_generic ON type_relationships(is_generic) WHERE is_generic = TRUE;

-- Compound indexes for type relationship queries
CREATE INDEX idx_type_relationships_from_rel_type ON type_relationships(from_type_id, relationship_type);
CREATE INDEX idx_type_relationships_to_rel_type ON type_relationships(to_type_id, relationship_type);

-- Full-text search indexes
CREATE INDEX idx_ast_elements_search ON ast_elements USING GIN(
    to_tsvector('english', 
        COALESCE(name, '') || ' ' || 
        COALESCE(qualified_name, '') || ' ' || 
        COALESCE(signature, '') || ' ' ||
        COALESCE(array_to_string(doc_comments, ' '), '')
    )
);

-- Partial indexes for performance optimization
CREATE INDEX idx_ast_elements_functions ON ast_elements(project_id, name) 
    WHERE element_type = 'Function';
CREATE INDEX idx_ast_elements_structs ON ast_elements(project_id, name) 
    WHERE element_type = 'Struct';
CREATE INDEX idx_ast_elements_traits ON ast_elements(project_id, name) 
    WHERE element_type = 'Trait';

-- Covering indexes for common read patterns
CREATE INDEX idx_ast_elements_summary ON ast_elements(project_id, file_id) 
    INCLUDE (element_type, name, visibility, complexity);

-- Update triggers for maintaining timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_projects_updated_at 
    BEFORE UPDATE ON projects 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ast_elements_updated_at 
    BEFORE UPDATE ON ast_elements 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Views for common queries
CREATE VIEW project_summary AS
SELECT 
    p.id,
    p.name,
    p.version,
    p.total_files,
    p.total_functions,
    p.total_structs,
    p.total_enums,
    p.total_traits,
    p.complexity_average,
    p.created_at,
    p.analyzed_at,
    COUNT(f.id) as analyzed_files
FROM projects p
LEFT JOIN files f ON p.id = f.project_id
GROUP BY p.id, p.name, p.version, p.total_files, p.total_functions, 
         p.total_structs, p.total_enums, p.total_traits, p.complexity_average,
         p.created_at, p.analyzed_at;

CREATE VIEW element_hierarchy AS
WITH RECURSIVE element_tree AS (
    -- Base case: root elements
    SELECT 
        id, project_id, name, qualified_name, element_type, 
        parent_element_id, nesting_level, 
        ARRAY[id] as path,
        0 as depth
    FROM ast_elements 
    WHERE parent_element_id IS NULL
    
    UNION ALL
    
    -- Recursive case: child elements
    SELECT 
        e.id, e.project_id, e.name, e.qualified_name, e.element_type,
        e.parent_element_id, e.nesting_level,
        et.path || e.id,
        et.depth + 1
    FROM ast_elements e
    JOIN element_tree et ON e.parent_element_id = et.id
)
SELECT * FROM element_tree;

CREATE VIEW call_graph AS
SELECT 
    cc.id,
    cc.project_id,
    caller.name as caller_name,
    caller.qualified_name as caller_qualified_name,
    callee.name as callee_name,
    callee.qualified_name as callee_qualified_name,
    cc.call_type,
    cc.call_count,
    cc.is_recursive,
    cc.recursion_depth
FROM call_chains cc
JOIN ast_elements caller ON cc.caller_id = caller.id
JOIN ast_elements callee ON cc.callee_id = callee.id;

-- Comments for documentation
COMMENT ON TABLE projects IS 'Main projects table storing Rust project metadata';
COMMENT ON TABLE files IS 'File-level information and metrics for each source file';
COMMENT ON TABLE ast_elements IS 'Individual AST elements (functions, structs, etc.) with location and complexity data';
COMMENT ON TABLE cross_references IS 'References between elements for dependency tracking';
COMMENT ON TABLE dependencies IS 'Processed dependency relationships between elements';
COMMENT ON TABLE call_chains IS 'Function call relationships for call graph analysis';
COMMENT ON TABLE type_relationships IS 'Type-level relationships (implements, extends, etc.)';

COMMENT ON INDEX idx_ast_elements_search IS 'Full-text search across element names, signatures, and documentation';
COMMENT ON INDEX idx_cross_references_from_element IS 'Critical for forward dependency traversal';
COMMENT ON INDEX idx_cross_references_to_element IS 'Critical for backward dependency traversal';
COMMENT ON INDEX idx_dependencies_from_element IS 'Optimizes dependency graph traversal from source';
COMMENT ON INDEX idx_dependencies_to_element IS 'Optimizes dependency graph traversal to target';