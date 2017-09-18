{
    "targets": [{
        "target_name": "neon",
        "sources": [ "src/neon.cc" ],
        "include_dirs": [ "<!(node -e \"require('nan')\")" ],
        'configurations': {
            'Release': {
                'msvs_settings': {
                    'VCCLCompilerTool': {
                        'WholeProgramOptimization': 'false'
                    },
                    'VCLinkerTool': {
                        'LinkTimeCodeGeneration': 0
                    }
                }
            }
        }
    }]
}
