## Vainilla Parser

Regla gramática del lenguaje:

```
programa —> main { lista_declaración }
listaDeclaración —> listaDeclaración declaración |declaración
declaración -> declaraciónVariable |listaSentencias
declaraciónVariable -> tipo identificador;
identificador -> identificador , id | id
tipo —> integer | double
listaSentencias —> listaSentencia sentencia |vacío
sentencia -> selección | iteración | repetición | sentIn |sentOut | asignación
asignación -> id = sentExpresión
sentExpresión -> expresión; | ;
selección -> if expresión { listaSentencias } |if expresión { listaSentencias } else { listaSentencias }
iteración -> while expresión { listaSentencias }
repetición -> do { listaSentencias } while expresión ;
sentIn -> cin id;
sentOut -> cout expresión;
expresión -> expresiónSimple relaciónOp expresiónSimple | expresiónSimple
relacionOp -> < |<= | > | >= | == | !=
expresiónSimple -> expresiónSimple sumaOp termino | termino
sumaOp -> + =
termino -> termino multOp factor | factor
multOp -> * | / |%
factor -> factor potOp componente | componente
multOp -> ^
componente -> ( expresión ) | número | incremento
incremento —> id operadorIncremento | id
operadorIncremento —> ++ | --
```

Para probar:

```
cargo run -- --verbose --save build assets/ejemplo.cat
```
