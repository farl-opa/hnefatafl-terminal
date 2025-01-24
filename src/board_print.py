import matplotlib.pyplot as plt
import numpy as np
import ast
import json
import matplotlib.animation as animation

# Función para procesar el estado del tablero desde una cadena
def create_board_from_string(board_string):
    board_state = ast.literal_eval(board_string)  # Convertir la cadena a un diccionario
    board = np.full((7, 7), 'White')  # Crear un tablero vacío

    # Colorear el tablero según el estado
    for position, piece in board_state.items():
        x, y = map(int, position.strip("()").split(","))
        
        if piece == 'Attacker':
            board[6 - x, y] = 'Black'  # Invertir la coordenada x
        elif piece in ['Defender']:
            board[6 - x, y] = 'Blue'  # Invertir la coordenada x
        elif piece in ['King']:
            board[6 - x, y] = 'Red'  # Invertir la coordenada x
    
    return board

def convert_board_to_numeric(board):
    color_map = {'White': 0, 'Black': 1, 'Blue': 2, 'Red': 3}
    numeric_board = np.vectorize(color_map.get)(board)
    return numeric_board

# Leer el archivo board_state.txt
with open('board_state.txt', 'r') as file:
    board_states = [json.loads(line)['board'] for line in file]

# Crear la figura para la animación
fig, ax = plt.subplots(figsize=(7, 7))

def update(frame):
    ax.clear()
    board = create_board_from_string(str(board_states[frame]))
    numeric_board = convert_board_to_numeric(board)
    ax.imshow(numeric_board, cmap='viridis', interpolation='nearest')
    ax.set_xticks(np.arange(7))
    ax.set_yticks(np.arange(7))
    ax.set_xticklabels(np.arange(7))
    ax.set_yticklabels(np.arange(7))
    ax.grid(True)

ani = animation.FuncAnimation(fig, update, frames=len(board_states), interval=500)

plt.show()