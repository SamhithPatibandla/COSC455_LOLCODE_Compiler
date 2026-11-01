grammar lolcode;

lolcode
    : HAI document_content KBYE EOF
    ;

document_content
    : variable_definition body_content
    | comment after_leading_comment
    | head body_content
    | body_start
    ;

after_leading_comment
    : comment after_leading_comment
    | head body_content
    | body_start
    ;

body_start
    : body_noncomment_element body_content
    |
    ;

head
    : HEAD_START head_content MKAY
    ;

head_content
    : comment head_content
    | title head_after_title
    |
    ;

head_after_title
    : comment head_after_title
    |
    ;

title
    : TITLE_START plain_text OIC
    ;

body_content
    : body_element body_content
    |
    ;

body_element
    : text
    | comment
    | paragraph
    | bold
    | list
    | newline
    | link
    | variable_use
    ;

body_noncomment_element
    : text
    | paragraph
    | bold
    | list
    | newline
    | link
    | variable_use
    ;

comment
    : COMMENT_START plain_text COMMENT_END
    ;

paragraph
    : PARAGRAPH_START paragraph_start MKAY
    ;

paragraph_start
    : variable_definition paragraph_content
    | paragraph_content
    ;

paragraph_content
    : paragraph_element paragraph_content
    |
    ;

paragraph_element
    : text
    | comment
    | bold
    | list
    | newline
    | link
    | variable_use
    ;

bold
    : BOLD_START bold_content OIC
    ;

bold_content
    : text bold_content
    | variable_use bold_content
    |
    ;

list
    : LIST_START list_items MKAY
    ;

list_items
    : item more_list_items
    ;

more_list_items
    : item more_list_items
    |
    ;

item
    : ITEM_START item_start OIC
    ;

item_start
    : variable_definition item_content
    | item_content
    ;

item_content
    : item_element item_content
    |
    ;

item_element
    : text
    | bold
    | variable_use
    ;

newline
    : NEWLINE
    ;

link
    : LINK_START address OIC
    ;

address
    : text_unit address_tail
    ;

address_tail
    : text_unit address_tail
    |
    ;

variable_definition
    : VAR_DEF_START variable_name ITIZ value MKAY
    ;

variable_use
    : VAR_USE_START variable_name OIC
    ;

variable_name
    : WORD
    ;

value
    : text_unit value_tail
    ;

value_tail
    : text_unit value_tail
    |
    ;

plain_text
    : text_unit plain_text_tail
    ;

plain_text_tail
    : text_unit plain_text_tail
    |
    ;

text
    : text_unit
    ;

text_unit
    : WORD
    | NUMBER
    | PUNCTUATION
    ;

HAI
    : '#' H A I
    ;

KBYE
    : '#' K B Y E
    ;

COMMENT_START
    : '#' O B T W
    ;

COMMENT_END
    : '#' T L D R
    ;

HEAD_START
    : '#' M A E K ' ' H E A D
    ;

TITLE_START
    : '#' G I M M E H ' ' T I T L E
    ;

PARAGRAPH_START
    : '#' M A E K ' ' P A R A G R A F
    ;

BOLD_START
    : '#' G I M M E H ' ' B O L D
    ;

LIST_START
    : '#' M A E K ' ' L I S T
    ;

ITEM_START
    : '#' G I M M E H ' ' I T E M
    ;

NEWLINE
    : '#' N E W L I N E
    ;

LINK_START
    : '#' G I M M E H ' ' L I N X
    ;

VAR_DEF_START
    : '#' I H A Z
    ;

ITIZ
    : '#' I T I Z
    ;

VAR_USE_START
    : '#' G I M M E H ' ' V A R
    ;

MKAY
    : '#' M K A Y
    ;

OIC
    : '#' O I C
    ;

WORD
    : ('a'..'z' | 'A'..'Z')+
    ;

NUMBER
    : ('0'..'9')+
    ;

PUNCTUATION
    : ','
    | '.'
    | '"'
    | ':'
    | '?'
    | '!'
    | '%'
    | '='
    | '/'
    ;
    
WS
    : (' ' | '\t' | '\r' | '\n')+ {skip();}
    ;
    
fragment A : 'a' | 'A';
fragment B : 'b' | 'B';
fragment C : 'c' | 'C';
fragment D : 'd' | 'D';
fragment E : 'e' | 'E';
fragment F : 'f' | 'F';
fragment G : 'g' | 'G';
fragment H : 'h' | 'H';
fragment I : 'i' | 'I';
fragment J : 'j' | 'J';
fragment K : 'k' | 'K';
fragment L : 'l' | 'L';
fragment M : 'm' | 'M';
fragment N : 'n' | 'N';
fragment O : 'o' | 'O';
fragment P : 'p' | 'P';
fragment Q : 'q' | 'Q';
fragment R : 'r' | 'R';
fragment S : 's' | 'S';
fragment T : 't' | 'T';
fragment U : 'u' | 'U';
fragment V : 'v' | 'V';
fragment W : 'w' | 'W';
fragment X : 'x' | 'X';
fragment Y : 'y' | 'Y';
fragment Z : 'z' | 'Z';