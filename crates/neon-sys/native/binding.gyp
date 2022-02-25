{
    "targets": [{
        "target_name": "neon",
        "sources": [ "src/neon.cc" ],
        "include_dirs": [ "<!(node -e \"require('nan')\")" ],
        'configurations': {
            'Release': {
                'msvs_settings': {
                    'VCCLCompilerTool': {
                        # Optimization adds the `/GL` flag which causes an undocumented
                        # private object file representation instead of a standard COFF
                        # that Rust can link.
                        'WholeProgramOptimization': 'false'
                    },
                    'VCLinkerTool': {
                        'LinkTimeCodeGeneration': 0
                    }
                }
            },
            'Debug': {
                'msvs_settings': {
                    'VCCLCompilerTool': {
                        'WholeProgramOptimization': 'false',
                        'RuntimeLibrary': '0',
                        'UndefinePreprocessorDefinitions': ['DEBUG', '_DEBUG'],
                        'PreprocessorDefinitions': ['NDEBUG']
                    },
                }
            }
        }
    }]
}
