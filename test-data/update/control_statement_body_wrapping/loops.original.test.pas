program BodyWrapFixture;

begin
  for I := 1 to 3 do
    Foo(I);

  for Value in Values do Bar(Value);

  for I := 1 to 3 do
    // note
    Foo(I);

  for I := 1 to 3 do Foo(I);

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
    Foo(I); // tail
end.
