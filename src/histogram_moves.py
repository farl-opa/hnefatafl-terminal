import json
import matplotlib.pyplot as plt
import matplotlib.animation as animation

def read_game_states(file_path):
    with open(file_path, 'r') as file:
        lines = file.readlines()
    moves = [int(line.strip()) for line in lines[1:]]  # Convert moves to integers
    return moves

if __name__ == "__main__":
    file_path = "defender_session_1_moves.txt"
    moves = read_game_states(file_path)
    
    # Sort the moves
    moves.sort()
    
    # Create histogram with bins
    plt.hist(moves, bins=20, edgecolor='black')
    
    # Set x-axis ticks to show fewer labels
    plt.xticks(rotation=45, ha='right')
    
    plt.xlabel('Moves')
    plt.ylabel('Frequency')
    plt.tight_layout()  # Adjust layout to prevent clipping of tick-labels
    plt.show()