# Kintampo

**Status:** Planning phase

Themes and thoughts:

- Kintampo is a system, not a library
- Initial implementation will use the file system for adding and organizing data
- Leverages hierarchy inherent in folders/files to define (simple) data flows
- Focused on interactive usage for powerful data visualization and comprehension
- Kintampo will have built-in data -> viz facilities leveraging standards like [Vega](https://vega.github.io/vega/examples/)
- Kintampo will support arbitrary scripts in any folder, so compiled languages can call executables and interpreted languages can invoke themselves using data dropped into the folder.
- Data processing is done asynchronously and taking advantage of multiple CPUs
- Uses ZeroMQ under the covers, and so can just as easily be entirely local or distributed one or more networks

Non-themes, since these are being tackled by folks working on Kafka, Flink, Spark, Storm, Onyx, NiFi, etc.:

- Big data
- High performance
- Streaming
- Pretty UI

## Design

Example use-cases:

- Visualization
   - There is a folder called `earthquakes`
   - There are subfolders `earthquakes/plot.bar.svg` and `earthquakes/plot.pie.svg`
   - Drop a `2018-01-01.csv` file of that date's earthquake data into the `earthquakes` folder
   - Immediately, the CSV data is realized as a bar chart under `earthquakes/plot.bar.svg/2018-01-01.csv.svg` and a pie chart under `earthquakes/plot.pie.svg/2018-01-01.csv.svg` in SVG format
   - There could be an intermediate folder like `earthquakes/viz/<name>` that would capture common operations or settings across multiple folders (e.g., to limit which fields in a data set are to be used, set graph options, etc.)

## License

Copyright 2018 Daniel Gregoire

All source code is licensed under the Mozilla Public License, Version 2.0.
