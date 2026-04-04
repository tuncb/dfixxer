program BodyWrapOptions;

begin
  for I := 1 to 3 do
    Exit;

  while Ready do
  begin
    Break;
  end;

  if LDone then
  begin
    Halt(1);
  end
  else
  begin
    Abort;
  end;
end.
