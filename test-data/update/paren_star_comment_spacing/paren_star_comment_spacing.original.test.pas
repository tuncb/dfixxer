program ParenStarCommentSpacing;
begin
  (*NoSpace*)
  (*  TooManySpaces   *)
  (*First line
Second line*)
  (*
Second line
  *)
  (*$IFDEF DEBUG*)
  Writeln('debug');
  (*$ENDIF*)
end.
