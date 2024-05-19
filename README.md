## Parser

```
programa —> main { lista_declaración }
listaDeclaración —> listaDeclaración declaración |declaración
declaración -> declaraciónVariable |listaSentencias
declaraciónVariable -> tipo identificador;
identificador -> identificador , id | id
tipo —> integer | double
listaSentencias —> listaSentencia sentencia |vacío
sentencia -> selección | iteración | repetición | sentIn |sentOut | asignación
asignación -> = sentExpresión
sentExpresión -> expresión; | ;
selección -> if expresión { sentencia } |if expresión { sentencia } else { sentencia } end
iteración -> while expresión { sentencia }
repetición -> do { sentencia } while expresión ;
sentIn -> cin id;
sentOut -> cout expresión;
expresión -> expresiónSimple relaciónOp expresiónSimple | expresiónSimple
relacionOp -> < |<= | > | >= | == | !=
expresiónSimple -> expresiónSimple sumaOp termino | termino
termino -> termino multOp factor | factor
multOp -> * | / |%
factor -> factor potOp componente | componente
componente -> ( expresión ) | número | incremento
incremento —> id operadorIncremento | id
operadorIncremento —> ++ | --
```
