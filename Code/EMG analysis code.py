#need to install "pandas" and "numpy" in computer terminal for EMG analysis

import pandas as pd
import numpy as np
import os
import re

# -----------------------------
# SETTINGS
# -----------------------------

#Change this filepath to you're designated location where the participant data files are stored
DATA_FOLDER = r"C:\Psychology\Masters\Training, Sensors and Simulation\TSS Data"  # folder where your files are stored

# Define electrode groups
#Change these electrode markers according to your own setup
MUSCLE_1 = [0, 5]
MUSCLE_2 = [6, 7]


# -----------------------------
# HELPER FUNCTIONS
# -----------------------------

def load_emg_file(filepath):
    """Load EMG file and keep only relevant columns."""
    df = pd.read_csv(filepath, sep=None, engine="python")

    df = df.iloc[:, [0, 3, 4]]
    df.columns = ["timestamp", "electrode", "value"]

    return df


def compute_muscle_stats(df):
    """Compute mean and std for each muscle."""
    muscle1_values = df[df["electrode"].isin(MUSCLE_1)]["value"]
    muscle2_values = df[df["electrode"].isin(MUSCLE_2)]["value"]

    stats = {
        "muscle1_mean": muscle1_values.mean(),
        "muscle1_std": muscle1_values.std(),
        "muscle2_mean": muscle2_values.mean(),
        "muscle2_std": muscle2_values.std(),
    }

    return stats


def compute_threshold_percentage(df, baseline_stats):
    """Calculate % of time BOTH muscles exceed baseline threshold simultaneously."""

    # Get thresholds
    m1_threshold = baseline_stats["muscle1_mean"] + 0.1 * baseline_stats["muscle1_std"]
    m2_threshold = baseline_stats["muscle2_mean"] + 0.1 * baseline_stats["muscle2_std"]

    # Compute mean per timestamp per muscle
    grouped = df.groupby(["timestamp", "electrode"])["value"].mean().reset_index()

    # Separate muscles
    muscle1 = grouped[grouped["electrode"].isin(MUSCLE_1)]
    muscle2 = grouped[grouped["electrode"].isin(MUSCLE_2)]

    # Average electrodes within each muscle per timestamp
    m1_time = muscle1.groupby("timestamp")["value"].mean()
    m2_time = muscle2.groupby("timestamp")["value"].mean()

    # Align timestamps
    combined = pd.DataFrame({
        "muscle1": m1_time,
        "muscle2": m2_time
    }).dropna()

    # Check condition: BOTH exceed threshold
    condition = (combined["muscle1"] > m1_threshold) & \
                (combined["muscle2"] > m2_threshold)

    percentage = (condition.sum() / len(combined)) * 100

    return {
        "both_muscles_percentage_exceeding": percentage
    }


def parse_filename(filename):
    """
    Extract participant and trial info from filename.
    Expected formats:
    - Participant 1 Baseline.yld
    - Participant 1 Trial 2.yld
    """
    baseline_match = re.match(r"Participant (\d+) Baseline", filename)
    trial_match = re.match(r"Participant (\d+) Trial (\d+)", filename)

    if baseline_match:
        return int(baseline_match.group(1)), "baseline", None
    elif trial_match:
        return int(trial_match.group(1)), "trial", int(trial_match.group(2))
    else:
        return None, None, None


# -----------------------------
# MAIN PIPELINE
# -----------------------------

def process_all_data():
    baselines = {}
    results = []

    for file in os.listdir(DATA_FOLDER):
        if not file.endswith(".yld"):
            continue

        filepath = os.path.join(DATA_FOLDER, file)
        participant, filetype, trial = parse_filename(file)

        if participant is None:
            continue

        df = load_emg_file(filepath)

        # -----------------------------
        # BASELINE PROCESSING
        # -----------------------------
        if filetype == "baseline":
            stats = compute_muscle_stats(df)
            baselines[participant] = stats

            print(f"Stored baseline for Participant {participant}")

        # -----------------------------
        # TRIAL PROCESSING
        # -----------------------------
        elif filetype == "trial":
            if participant not in baselines:
                print(f"No baseline for Participant {participant}, skipping...")
                continue

            baseline_stats = baselines[participant]

            stats = compute_muscle_stats(df)
            threshold_results = compute_threshold_percentage(df, baseline_stats)

            result_entry = {
                "participant": participant,
                "trial": trial,
                **stats,
                **threshold_results,
            }

            results.append(result_entry)

    return pd.DataFrame(results)


# -----------------------------
# RUN
# -----------------------------

if __name__ == "__main__":
    final_results = process_all_data()

    print("\nFinal Results:")
    print(final_results)

    # Save results to .csv file
    final_results.to_csv("emg_analysis_results.csv", index=False)