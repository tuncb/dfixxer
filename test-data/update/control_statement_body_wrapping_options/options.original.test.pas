program BodyWrapOptions;

begin
  for I := 1 to 3 do
    Exit;

  while Ready do
    Break;

  if LDone then
    Halt(1)
  else
    Abort;
end.
