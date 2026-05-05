IMAGE  ?= localhost/tw3-replay-dev-toolbox
TAG    ?= latest
NAME   ?= tw3-replay-dev

build:
	podman build -t $(IMAGE):$(TAG) -f Containerfile .

create:
	toolbox create --image $(IMAGE):$(TAG) $(NAME)

enter:
	toolbox enter $(NAME)

stop:
	-toolbox run -c $(NAME) emacsclient --eval '(kill-emacs)' 2>/dev/null
	-podman stop $(NAME)

rm: stop
	toolbox rm $(NAME)

.PHONY: build create enter stop rm
