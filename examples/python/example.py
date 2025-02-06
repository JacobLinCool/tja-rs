# Run this example with `python examples/python/example.py`


def main():
    import json
    from tja import parse_tja

    # Read the TJA file content
    tja_file = "data/SUPERNOVA.tja"
    with open(tja_file, "r") as f:
        tja_content = f.read()

    # Parse the TJA content; autocompletion available on the parsed object
    parsed = parse_tja(tja_content)

    # Print the metadata
    print(f"Metadata: {parsed.metadata}")

    # Extract the chart for course "Ura"
    ura_chart = next((chart for chart in parsed.charts if chart.course == "Ura"), None)
    if not ura_chart:
        print("Ura course chart not found.")
        return

    # Print basic details of the Ura chart
    print(ura_chart.course, ura_chart.level, ura_chart.balloons)

    # Print the number of segments and details of the first segment
    print(f"Number of segments: {len(ura_chart.segments)}")
    print("First segment:", ura_chart.segments[0])

    # Export the data of tja classes to a dictionary containing only primitive types
    first_segment = ura_chart.segments[0].export()
    print("Full Ura chart export:\n", json.dumps(first_segment, indent=4))


if __name__ == "__main__":
    main()
