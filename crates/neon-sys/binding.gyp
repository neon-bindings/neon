{
  "targets": [{
    "target_name": "neon",
    "sources": [ "src/neon.cc" ],
    "include_dirs": [ "<!(node -e \"require('nan')\")" ]
  }]
}
