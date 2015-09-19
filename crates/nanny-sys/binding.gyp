{
  "targets": [{
    "target_name": "nanny",
    "sources": [ "src/nanny.cc" ],
    "include_dirs": [ "<!(node -e \"require('nan')\")" ]
  }]
}
