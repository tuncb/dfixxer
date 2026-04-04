program BodyWrapFixture;

begin
  for I := 1 to 3 do
  begin
    Foo(I);
  end;

  while Ready do
  begin
    Step;
  end;

  for Value in Values do
  begin
    Bar(Value);
  end;

  for I := 1 to 3 do
  begin
    // note
    Foo(I);
  end;

  while Ready do
  begin
    // note
    Step;
  end;

  for I := 1 to 3 do
  begin
    Foo(I);
  end;

  while Ready do
  begin
    Step;
  end;

  for I := 1 to 3 do
  begin
    AlreadyWrapped;
  end;

  while Ready do
  begin
    AlreadyWrapped;
  end;

  for I := 1 to 3 do
    Exit;
  for I := 1 to 3 do
    EXIT(1);
  for I := 1 to 3 do
    Continue;
  for I := 1 to 3 do
    Break;
  for I := 1 to 3 do
    raise Exception.Create('boom');
  for I := 1 to 3 do
    Abort;
  for I := 1 to 3 do
    Halt(1);
  for I := 1 to 3 do
    ;
  while Ready do
    Exit;
  while Ready do
    Continue;
  while Ready do
    Break;
  while Ready do
    raise Exception.Create('boom');
  while Ready do
    Abort;
  while Ready do
    Halt(1);
  while Ready do
    ;
  for I := 1 to 3 do
  begin
    Foo(I); // tail
  end;
  while Ready do
  begin
    Step; // tail
  end;
end.
