program BodyWrapFixture;

begin
  for I := 1 to 3 do
  begin
    Foo(I);
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

  for I := 1 to 3 do
  begin
    Foo(I);
  end;

  for I := 1 to 3 do
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
  for I := 1 to 3 do
  begin
    Foo(I); // tail
  end;
end.
