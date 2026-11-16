window.BENCHMARK_DATA = {
  "lastUpdate": 1794853168625,
  "repoUrl": "https://github.com/zhukoff74/strverify",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "zhukoffwork@gmail.com",
            "name": "Dmitry Zhukov",
            "username": "zhukoff74"
          },
          "committer": {
            "email": "zhukoffwork@gmail.com",
            "name": "Dmitry Zhukov",
            "username": "zhukoff74"
          },
          "distinct": true,
          "id": "061857ca84ffaa9ffd690f74fa456907d1648c3b",
          "message": "hot fix banchmark",
          "timestamp": "2026-11-16T21:08:53+03:00",
          "tree_id": "48160c207b8c27f7ae022b18155ff08e18fc9084",
          "url": "https://github.com/zhukoff74/strverify/commit/061857ca84ffaa9ffd690f74fa456907d1648c3b"
        },
        "date": 1794852580699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "only_cyrillic",
            "value": 0.563,
            "unit": "us"
          },
          {
            "name": "only_latin",
            "value": 0.496,
            "unit": "us"
          },
          {
            "name": "only_digits",
            "value": 0.289,
            "unit": "us"
          },
          {
            "name": "only_digits_and",
            "value": 0.445,
            "unit": "us"
          },
          {
            "name": "contains",
            "value": 0.409,
            "unit": "us"
          },
          {
            "name": "compose_and_not",
            "value": 1.473,
            "unit": "us"
          },
          {
            "name": "long_text_cyrillic",
            "value": 9.558,
            "unit": "us"
          },
          {
            "name": "long_text_no_match",
            "value": 0.695,
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "zhukoffwork@gmail.com",
            "name": "Dmitry Zhukov",
            "username": "zhukoff74"
          },
          "committer": {
            "email": "zhukoffwork@gmail.com",
            "name": "Dmitry Zhukov",
            "username": "zhukoff74"
          },
          "distinct": true,
          "id": "c9719a71731aa77966752d3e829e49ecaf31ccaa",
          "message": "Update README.md",
          "timestamp": "2026-11-16T21:18:48+03:00",
          "tree_id": "6f6a481ed0ab6605d1a7fc64c4359f42a5940913",
          "url": "https://github.com/zhukoff74/strverify/commit/c9719a71731aa77966752d3e829e49ecaf31ccaa"
        },
        "date": 1794853166937,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "only_cyrillic",
            "value": 0.565,
            "unit": "us"
          },
          {
            "name": "only_latin",
            "value": 0.503,
            "unit": "us"
          },
          {
            "name": "only_digits",
            "value": 0.263,
            "unit": "us"
          },
          {
            "name": "only_digits_and",
            "value": 0.403,
            "unit": "us"
          },
          {
            "name": "contains",
            "value": 0.383,
            "unit": "us"
          },
          {
            "name": "compose_and_not",
            "value": 1.482,
            "unit": "us"
          },
          {
            "name": "long_text_cyrillic",
            "value": 8.368,
            "unit": "us"
          },
          {
            "name": "long_text_no_match",
            "value": 0.711,
            "unit": "us"
          }
        ]
      }
    ]
  }
}