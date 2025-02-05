import json
import matplotlib.pyplot as plt
import matplotlib.animation as animation

# Define colors for different pieces
colors = {
    "Empty": "white",
    "Attacker": "red",
    "Defender": "blue",
    "King": "gold"
}

def read_game_states(file_path):
    with open(file_path, 'r') as file:
        lines = file.readlines()
    game_states = [json.loads(line.strip()) for line in lines[:-1]]  # Exclude the last line (winner)
    return game_states

def plot_board(board, ax):
    ax.clear()
    ax.set_xticks(range(8))
    ax.set_yticks(range(8))
    ax.set_xticklabels([])
    ax.set_yticklabels([])
    ax.grid(True)
    ax.set_aspect('equal')  # Ensure each cell is a perfect square
    
    for key, piece in board.items():
        x, y = eval(key)  # Convert string key to tuple
        ax.add_patch(plt.Rectangle((x, y), 1, 1, color=colors[piece]))
    
    # Add legend
    handles = [plt.Rectangle((0, 0), 1, 1, color=colors[piece]) for piece in colors]
    labels = list(colors.keys())
    ax.legend(handles, labels, loc='upper right', bbox_to_anchor=(1.25, 1))

def animate(i, game_states, ax):
    plot_board(game_states[i]["board"], ax)

def main(file_path):
    game_states = read_game_states(file_path)
    
    fig, ax = plt.subplots(figsize=(10, 8))
    ani = animation.FuncAnimation(fig, animate, frames=len(game_states), fargs=(game_states, ax), interval=500)
    
    # Set title
    ax.set_title('Hnefatafl Game Animation')
    
    plt.show()

if __name__ == "__main__":
    file_path = "board_state_game_1_2.txt"
    main(file_path)