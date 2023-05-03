# Correction de la faille

## Explication de la faille

La faille est presente dans la methode updateProgress de la classe TaskController. Cette methode permet de mettre a jour la progression d'une tache. Cependant, elle ne verifie pas que la tache appartient bien a l'utilisateur connecte. Ainsi, un utilisateur peut mettre a jour la progression d'une tache qui ne lui appartient pas.

## Etapes de reproduction

1. Se connecter avec l'utilisateur "user1" et le mot de passe "user1"
2. Trouver l'id de la tache en essayant plusieurs id dans la methode detail de la classe TaskController.
3. Mettre a jour la progression de la tache avec l'id trouve a l'etape 2 dans la methode updateProgress de la classe TaskController.

## Etapes de correction

Pour reparer la faile, il faudrait verifier que la tache appartient bien a l'utilisateur connecte. Il faudrait faire cela sur la methode updateProgress de la classe TaskController, mais idealement aussi sur la methode detail pour empecher les utilisateurs de voir les taches qui ne leurs appartiennent pas.

## Example de code

```Java
 @Override
    public TaskDetailResponse detail(Long id, MUser user) {
        MTask element = user.tasks.stream().filter(elt -> elt.id == id).findFirst().get(); // on prend la tache seulement si elle appartient a l'utilisateur
        TaskDetailResponse response = new TaskDetailResponse();
        response.name = element.name;
        response.id = element.id;
        // calcul le temps écoulé en pourcentage
        response.percentageTimeSpent = percentage(element.creationDate, new Date(), element.deadline);
        // aller chercher le dernier événement de progrès
        response.percentageDone = percentageDone(element);
        response.deadline = element.deadline;
        response.events = new ArrayList<>();
        for (MProgressEvent e : element.events) {
            ProgressEvent transfer = new ProgressEvent();
            transfer.value = e.resultPercentage;
            transfer.timestamp = e.timestamp;
            response.events.add(transfer);
        }
        return response;
    }

    @Override
    public void updateProgress(long taskID, int value) {
        MTask element = user.tasks.stream().filter(elt -> elt.id == id).findFirst().get(); // on prend la tache seulement si elle appartient a l'utilisateur
        // TODO validate value is between 0 and 100
        MProgressEvent pe= new MProgressEvent();
        pe.resultPercentage = value;
        pe.completed = value ==100;
        pe.timestamp = DateTime.now().toDate();
        repoProgressEvent.save(pe);
        element.events.add(pe);
        repo.save(element);
    }
```
