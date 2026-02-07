unit Repro_ForVarIn_Foreach;

interface

implementation

function CountItems(const AItems: TArray<Integer>): Integer;
begin
  // Finding: parser supports "for X in Y do" but not "for var X in Y do".
  // The "var" in foreach iteration causes ERROR recovery in tree-sitter-pascal 0.10.0.
  for var LItem in AItems do
  begin
    Inc(Result);
  end;
end;

end.
