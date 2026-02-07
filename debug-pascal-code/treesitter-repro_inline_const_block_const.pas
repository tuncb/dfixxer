unit Repro_InlineConst_BlockConst;

interface

implementation

function HasInlineConst: Integer;
begin
  // Finding: tree-sitter-pascal 0.10.0 treats block-local "const" as an ERROR node.
  // The grammar's block rule accepts statements, but not local const declarations.
  const LValue = 42;
  Result := LValue;
end;

end.
