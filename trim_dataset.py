import glob
import pathlib
import argparse
import os
from evalpy import TrackObject, MidiObject, trim_file



parser = argparse.ArgumentParser(description="Trim a set of midi files.")

parser.add_argument(
    "-i",
    dest="input_path",
    default="data/default/",
    help='Sets the input path (default: "data/default/")',
)

parser.add_argument(
    "-o",
    dest="output_path",
    default="data/quantized/",
    help='Sets the output path (default: "data/quantized/")',
)

def get_out_path(filepath, out_dir):
        basename = os.path.basename(filepath)
        if not os.path.exists(out_dir):
            os.makedirs(out_dir)

        return str(out_dir) + "/" + basename + ".trim.mid"
        

args = parser.parse_args().__dict__

data_dir = pathlib.Path(args["input_path"])
out_dir = pathlib.Path(args["output_path"])
files = glob.glob(str(data_dir / "**/*.mid*"))

for file in files:
       trim_file(
              file,
              get_out_path(file, out_dir)
       )
       