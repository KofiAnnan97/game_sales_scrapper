import sys

def add_double_slash_to_path(dir_path: str) -> str:
    new_dir_path = ""
    for i in range(len(dir_path)):
        if dir_path[i] == '\\' and (dir_path[i+1] != '\\' and dir_path[i-1] != '\\'):
            new_dir_path += r"\\"
        else:
            new_dir_path += dir_path[i]
    return new_dir_path

def main():
    if len(sys.argv) == 2:
        dir_path = sys.argv[1]
        print(add_double_slash_to_path(dir_path))
    else:
        print("This script should be run with the one argument, the filepath as a string")

if __name__ == '__main__':
    main()