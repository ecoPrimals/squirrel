---
title: "Galaxy Tool Definition Schema for MCP"
description: "Schema specification for representing Galaxy tools as MCP tool definitions"
version: "0.1.0"
last_updated: "2025-03-25"
status: "draft"
owners:
  primary: ["DataScienceBioLab", "mcp-team"]
  reviewers: ["core-team", "integration-team"]
---

# Galaxy Tool Definition Schema for MCP

## 1. Overview

This specification defines the schema for representing Galaxy tools in the Machine Context Protocol (MCP) format. It details how Galaxy's tool XML definitions are translated into MCP tool definitions, including parameter mapping, capabilities, and metadata.

## 2. Schema Definition

The Galaxy-MCP tool definition uses the following JSON schema:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Galaxy MCP Tool Definition",
  "type": "object",
  "required": ["id", "name", "version", "description", "capabilities"],
  "properties": {
    "id": {
      "type": "string",
      "description": "Unique identifier for the tool in MCP, prefixed with 'galaxy-tool-'",
      "pattern": "^galaxy-tool-[a-zA-Z0-9_\\-\\.]+$"
    },
    "name": {
      "type": "string",
      "description": "Human-readable name of the tool"
    },
    "version": {
      "type": "string",
      "description": "Semantic version of the tool",
      "pattern": "^[0-9]+(\\.[0-9]+)*(\\+[a-zA-Z0-9\\.]+)?$"
    },
    "description": {
      "type": "string",
      "description": "Comprehensive description of the tool's purpose and functionality"
    },
    "capabilities": {
      "type": "array",
      "description": "List of capabilities provided by the tool",
      "minItems": 1,
      "items": {
        "$ref": "#/definitions/capability"
      }
    },
    "securityLevel": {
      "type": "integer",
      "description": "Required security level (0-10)",
      "minimum": 0,
      "maximum": 10
    },
    "metadata": {
      "type": "object",
      "description": "Additional tool-specific metadata",
      "properties": {
        "category": {
          "type": "string",
          "description": "Galaxy tool category"
        },
        "galaxy_tool_id": {
          "type": "string",
          "description": "Original Galaxy tool ID"
        },
        "input_formats": {
          "type": "array",
          "description": "Supported input file formats",
          "items": {
            "type": "string"
          }
        },
        "output_formats": {
          "type": "array",
          "description": "Produced output file formats",
          "items": {
            "type": "string"
          }
        },
        "citations": {
          "type": "array",
          "description": "Academic citations for the tool",
          "items": {
            "type": "string"
          }
        },
        "requirements": {
          "type": "array",
          "description": "Tool dependencies",
          "items": {
            "type": "object",
            "properties": {
              "name": {
                "type": "string"
              },
              "version": {
                "type": "string"
              },
              "type": {
                "type": "string"
              }
            }
          }
        },
        "help_text": {
          "type": "string",
          "description": "Full help text from Galaxy tool"
        }
      }
    }
  },
  "definitions": {
    "capability": {
      "type": "object",
      "required": ["name", "description"],
      "properties": {
        "name": {
          "type": "string",
          "description": "Capability identifier"
        },
        "description": {
          "type": "string",
          "description": "Description of what the capability does"
        },
        "parameters": {
          "type": "array",
          "description": "Input parameters for the capability",
          "items": {
            "$ref": "#/definitions/parameter"
          }
        },
        "return": {
          "type": "object",
          "description": "Return value description and schema",
          "properties": {
            "description": {
              "type": "string"
            },
            "schema": {
              "type": "object"
            }
          }
        }
      }
    },
    "parameter": {
      "type": "object",
      "required": ["name", "description", "type"],
      "properties": {
        "name": {
          "type": "string",
          "description": "Parameter name"
        },
        "description": {
          "type": "string",
          "description": "Parameter description"
        },
        "type": {
          "type": "string",
          "description": "Parameter data type",
          "enum": ["string", "number", "boolean", "object", "array", "any"]
        },
        "required": {
          "type": "boolean",
          "description": "Whether the parameter is required"
        },
        "default": {
          "description": "Default value for the parameter"
        },
        "enum": {
          "type": "array",
          "description": "List of possible values for select parameters"
        },
        "format": {
          "type": "string",
          "description": "Format specification for special types (e.g., data_id)"
        },
        "properties": {
          "type": "object",
          "description": "Properties for object parameters"
        },
        "items": {
          "description": "Schema for array items"
        }
      }
    }
  }
}
```

## 3. Transformation Rules

### 3.1 Tool Metadata Mapping

| Galaxy XML Element | MCP Tool Property | Transformation Rule |
|-------------------|-------------------|---------------------|
| `<tool id="...">` | `id` | Prefix with "galaxy-tool-" |
| `<tool name="...">` | `name` | Direct mapping |
| `<tool version="...">` | `version` | Direct mapping |
| `<description>` | `description` | Direct mapping |
| `<tool profile="...">` | `metadata.galaxy_profile` | Direct mapping |
| `<tool>` section | `securityLevel` | Default to 5, adjustable based on capabilities |
| `<help>` | `metadata.help_text` | Convert Galaxy-specific markup to plain text |
| Tool panel section | `metadata.category` | Use Galaxy tool panel section |
| `<requirements>` | `metadata.requirements` | Map each requirement to an object |
| `<citations>` | `metadata.citations` | Extract citation text |

### 3.2 Parameter Mapping

| Galaxy Parameter Type | MCP Parameter Type | Notes |
|-----------------------|-------------------|-------|
| `type="text"` | `type: "string"` | Text inputs |
| `type="integer"` | `type: "number"` | Integer inputs |
| `type="float"` | `type: "number"` | Floating point inputs |
| `type="boolean"` | `type: "boolean"` | Boolean (checkbox) inputs |
| `type="select"` | `type: "string"`, with `enum` | Selection from options |
| `type="data"` | `type: "string"`, with `format: "data_id"` | References to datasets |
| `type="data_collection"` | `type: "array"`, with `items.format: "data_id"` | Collections of datasets |
| `type="drill_down"` | `type: "object"` | Hierarchical selectors |
| `type="color"` | `type: "string"` | Color picker values |

### 3.3 Conditional Parameters

Galaxy conditional parameters are mapped to nested object structures in MCP:

```json
// Galaxy conditional parameter in XML
<conditional name="advanced_options">
  <param name="show_advanced" type="boolean" label="Show advanced options" />
  <when value="true">
    <param name="min_quality" type="integer" value="20" label="Minimum quality score" />
  </when>
</conditional>

// Transformed to MCP parameter
{
  "name": "advanced_options",
  "description": "Advanced options configuration",
  "type": "object",
  "properties": {
    "show_advanced": {
      "description": "Show advanced options",
      "type": "boolean",
      "required": true
    },
    "min_quality": {
      "description": "Minimum quality score",
      "type": "number",
      "required": false,
      "default": 20,
      "condition": {
        "field": "show_advanced",
        "value": true
      }
    }
  }
}
```

### 3.4 Repeat Parameters

Galaxy repeat parameters are mapped to array types in MCP:

```json
// Galaxy repeat parameter in XML
<repeat name="treatments" title="Treatments">
  <param name="treatment_name" type="text" label="Treatment name" />
  <param name="concentration" type="float" label="Concentration" />
</repeat>

// Transformed to MCP parameter
{
  "name": "treatments",
  "description": "Treatments",
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "treatment_name": {
        "description": "Treatment name",
        "type": "string",
        "required": true
      },
      "concentration": {
        "description": "Concentration",
        "type": "number",
        "required": true
      }
    }
  }
}
```

## 4. Standard Capabilities

Every Galaxy tool is mapped to an MCP tool with at least the following capabilities:

### 4.1 Execute Capability

```json
{
  "name": "execute",
  "description": "Execute the Galaxy tool with specified parameters",
  "parameters": [
    {
      "name": "inputs",
      "description": "Input parameters for the tool",
      "type": "object",
      "required": true,
      "properties": {
        // Mapped from Galaxy tool parameters
      }
    },
    {
      "name": "history_id",
      "description": "Galaxy history ID to use",
      "type": "string",
      "required": false
    }
  ],
  "return": {
    "description": "Execution results including output datasets",
    "schema": {
      "type": "object",
      "properties": {
        "outputs": {
          "type": "object"
        },
        "job_info": {
          "type": "object"
        }
      }
    }
  }
}
```

### 4.2 Get Help Capability

```json
{
  "name": "get_help",
  "description": "Get detailed help for the tool",
  "parameters": [],
  "return": {
    "description": "Tool help information",
    "schema": {
      "type": "object",
      "properties": {
        "help_text": {
          "type": "string",
          "description": "Formatted help text"
        },
        "citations": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "requirements": {
          "type": "array",
          "items": {
            "type": "object"
          }
        }
      }
    }
  }
}
```

### 4.3 Get Form Capability

```json
{
  "name": "get_form",
  "description": "Get the form structure for the tool",
  "parameters": [],
  "return": {
    "description": "Tool form structure",
    "schema": {
      "type": "object",
      "properties": {
        "inputs": {
          "type": "array",
          "items": {
            "type": "object"
          }
        },
        "sections": {
          "type": "array",
          "items": {
            "type": "object"
          }
        }
      }
    }
  }
}
```

## 5. Examples

### 5.1 Simple Tool Example

**Galaxy Tool XML:**

```xml
<tool id="fastqc" name="FastQC" version="0.73+galaxy1" profile="21.05">
    <description>Read Quality reports</description>
    <requirements>
        <requirement type="package" version="0.11.9">fastqc</requirement>
    </requirements>
    <command>fastqc --outdir '${outdir}' '$input_file'</command>
    <inputs>
        <param name="input_file" type="data" format="fastq,fastq.gz,bam,sam" label="Short read data from your current history" />
    </inputs>
    <outputs>
        <data name="html_file" format="html" from_work_dir="*.html" label="${tool.name} on ${on_string}: Webpage" />
        <data name="text_file" format="txt" from_work_dir="*.txt" label="${tool.name} on ${on_string}: RawData" />
    </outputs>
    <help>
        FastQC aims to provide a simple way to do quality control checks on raw sequence data.
    </help>
</tool>
```

**Corresponding MCP Tool Definition:**

```json
{
  "id": "galaxy-tool-fastqc",
  "name": "FastQC",
  "version": "0.73+galaxy1",
  "description": "Read Quality reports",
  "capabilities": [
    {
      "name": "execute",
      "description": "Execute the FastQC tool with specified parameters",
      "parameters": [
        {
          "name": "inputs",
          "description": "Input parameters for the tool",
          "type": "object",
          "required": true,
          "properties": {
            "input_file": {
              "description": "Short read data from your current history",
              "type": "string",
              "format": "data_id",
              "required": true
            }
          }
        },
        {
          "name": "history_id",
          "description": "Galaxy history ID to use",
          "type": "string",
          "required": false
        }
      ],
      "return": {
        "description": "Execution results including output datasets",
        "schema": {
          "type": "object",
          "properties": {
            "outputs": {
              "type": "object",
              "properties": {
                "html_file": {
                  "type": "string",
                  "format": "data_id",
                  "description": "HTML report"
                },
                "text_file": {
                  "type": "string",
                  "format": "data_id",
                  "description": "Raw quality metrics data"
                }
              }
            },
            "job_info": {
              "type": "object"
            }
          }
        }
      }
    },
    {
      "name": "get_help",
      "description": "Get detailed help for the tool",
      "parameters": [],
      "return": {
        "description": "Tool help information",
        "schema": {
          "type": "object",
          "properties": {
            "help_text": {
              "type": "string"
            }
          }
        }
      }
    }
  ],
  "securityLevel": 3,
  "metadata": {
    "category": "FASTQ Quality Control",
    "galaxy_tool_id": "fastqc",
    "galaxy_profile": "21.05",
    "input_formats": ["fastq", "fastq.gz", "bam", "sam"],
    "output_formats": ["html", "txt"],
    "requirements": [
      {
        "name": "fastqc",
        "version": "0.11.9",
        "type": "package"
      }
    ],
    "help_text": "FastQC aims to provide a simple way to do quality control checks on raw sequence data."
  }
}
```

### 5.2 Complex Tool Example

**Galaxy Tool XML (Partial):**

```xml
<tool id="macs2_callpeak" name="MACS2 callpeak" version="2.2.7.1+galaxy0">
    <description>Call peaks from alignment results</description>
    <requirements>
        <requirement type="package" version="2.2.7.1">macs2</requirement>
        <requirement type="package" version="3.7">python</requirement>
    </requirements>
    <inputs>
        <conditional name="experiment_type">
            <param name="experiment_type_selector" type="select" label="Experiment type">
                <option value="ChIP-Seq" selected="true">ChIP-Seq</option>
                <option value="DNase-Seq">DNase-Seq</option>
            </param>
            <when value="ChIP-Seq">
                <param name="input_control_file" type="data" format="bam,sam" label="Control File" />
            </when>
            <when value="DNase-Seq">
                <!-- DNase-specific parameters -->
            </when>
        </conditional>
        <param name="treatment_file" type="data" format="bam,sam" label="Treatment File" />
        <param name="effective_genome_size" type="select" label="Effective genome size">
            <option value="2.7e9">Human (2.7e9)</option>
            <option value="1.87e9">Mouse (1.87e9)</option>
            <option value="1.4e8">Fly (1.4e8)</option>
        </param>
        <repeat name="replicates" title="Replicates">
            <param name="rep_treatment_file" type="data" format="bam,sam" label="Treatment File" />
        </repeat>
    </inputs>
    <!-- outputs and other elements omitted -->
</tool>
```

**Corresponding MCP Tool Definition (Partial):**

```json
{
  "id": "galaxy-tool-macs2_callpeak",
  "name": "MACS2 callpeak",
  "version": "2.2.7.1+galaxy0",
  "description": "Call peaks from alignment results",
  "capabilities": [
    {
      "name": "execute",
      "description": "Execute the MACS2 callpeak tool with specified parameters",
      "parameters": [
        {
          "name": "inputs",
          "description": "Input parameters for the tool",
          "type": "object",
          "required": true,
          "properties": {
            "experiment_type": {
              "description": "Experiment type configuration",
              "type": "object",
              "required": true,
              "properties": {
                "experiment_type_selector": {
                  "description": "Experiment type",
                  "type": "string",
                  "enum": ["ChIP-Seq", "DNase-Seq"],
                  "default": "ChIP-Seq",
                  "required": true
                },
                "input_control_file": {
                  "description": "Control File",
                  "type": "string",
                  "format": "data_id",
                  "required": true,
                  "condition": {
                    "field": "experiment_type_selector",
                    "value": "ChIP-Seq"
                  }
                }
              }
            },
            "treatment_file": {
              "description": "Treatment File",
              "type": "string",
              "format": "data_id",
              "required": true
            },
            "effective_genome_size": {
              "description": "Effective genome size",
              "type": "string",
              "enum": ["2.7e9", "1.87e9", "1.4e8"],
              "enum_labels": {
                "2.7e9": "Human (2.7e9)",
                "1.87e9": "Mouse (1.87e9)",
                "1.4e8": "Fly (1.4e8)"
              },
              "required": true
            },
            "replicates": {
              "description": "Replicates",
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "rep_treatment_file": {
                    "description": "Treatment File",
                    "type": "string",
                    "format": "data_id",
                    "required": true
                  }
                }
              }
            }
          }
        },
        {
          "name": "history_id",
          "description": "Galaxy history ID to use",
          "type": "string",
          "required": false
        }
      ],
      "return": {
        "description": "Execution results including output datasets",
        "schema": {
          "type": "object",
          "properties": {
            "outputs": {
              "type": "object"
            },
            "job_info": {
              "type": "object"
            }
          }
        }
      }
    }
  ],
  "securityLevel": 4,
  "metadata": {
    "category": "Peak Calling",
    "galaxy_tool_id": "macs2_callpeak",
    "input_formats": ["bam", "sam"],
    "requirements": [
      {
        "name": "macs2",
        "version": "2.2.7.1",
        "type": "package"
      },
      {
        "name": "python",
        "version": "3.7",
        "type": "package"
      }
    ]
  }
}
```

## 6. Related Specifications

- [Galaxy Tool XML Schema](https://docs.galaxyproject.org/en/latest/dev/schema.html)
- [MCP Tool Definition Specification](../mcp/protocol/tool-definition.md)
- [Galaxy API Mapping](api-mapping.md)
- [Galaxy MCP Integration Plan](galaxy-mcp-integration.md)

<version>0.1.0</version> 