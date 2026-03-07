unit InlineSuppressionAliases;

interface

procedure BraceDisabled;
procedure StarDisabled;
procedure EnabledAliasProc;

implementation

{dfixxer:off}
procedure BraceDisabled;
begin
  value:=1+2;
end;
{dfixxer:on}

(*dfixxer:off*)
procedure StarDisabled;
begin
  value:=3+4;
end;
(*dfixxer:on*)

procedure EnabledAliasProc;
begin
  value:=5+6;
end;

end.
